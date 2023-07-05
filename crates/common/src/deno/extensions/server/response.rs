use super::errors::{self};
use super::websocket::WebsocketStream;
use anyhow::{anyhow, Result};
use axum::response::sse::{Event, KeepAlive};
use axum::response::{sse::Sse, IntoResponse, Response};
use bytes::Bytes;
use deno_core::{ByteString, Resource, StringOrBuffer};
use http::header::{CONTENT_TYPE, UPGRADE};
use http::{HeaderName, HeaderValue};
use http_body::combinators::UnsyncBoxBody as HttpUnsyncBoxBody;
use http_body::Body;
use hyper::Body as HyperBody;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio_stream::wrappers::ReceiverStream;

pub type UnsyncedBoxBody = HttpUnsyncBoxBody<Bytes, errors::Error>;

pub trait IntoHttpResponse: IntoResponse {
  fn into_response(self) -> axum::response::Response;
}

#[derive(Debug, Default)]
pub struct ParsedHttpResponse {
  pub rid: u32,
  pub status: u16,
  pub headers: Vec<(ByteString, ByteString)>,
  pub data: Option<StringOrBuffer>,
  pub stream: Option<ReceiverStream<Result<Event>>>,
  pub websocket_tx: Option<oneshot::Sender<WebsocketStream>>,
}

pub struct HttpResponseMetata {
  pub method: String,
  pub path: String,
  pub req_received_at: Instant,
}

impl ParsedHttpResponse {
  pub fn has_upgrade_header(&self) -> bool {
    self.get_header(UPGRADE).is_some()
  }

  pub fn get_header(&self, header: HeaderName) -> Option<&[u8]> {
    self
      .headers
      .iter()
      .find(|(name, _)| {
        HeaderName::from_bytes(name)
          .map(|h| h == header)
          .unwrap_or(false)
      })
      .map(|(_, v)| v.as_ref())
  }

  pub async fn into_response(self) -> Result<Response, errors::Error> {
    let mut response_builder = Response::builder().status(self.status);
    for header in &self.headers {
      response_builder = response_builder.header(
        HeaderName::from_bytes(&header.0)?,
        HeaderValue::from_bytes(&header.1)?,
      );
    }

    match self.data {
      Some(data) => Ok(
        response_builder.body(
          HyperBody::from(Bytes::from(data.to_vec()))
            .map_err(|e| axum::Error::new(e))
            .boxed_unsync(),
        )?,
      ),
      None if self.stream.is_some() => {
        if self
          .get_header(CONTENT_TYPE)
          .map(|h| h != mime::TEXT_EVENT_STREAM.as_ref().as_bytes())
          .unwrap_or(true)
        {
          return Err(
            anyhow!(
              "Stream is only supported when content-type is {}",
              mime::TEXT_EVENT_STREAM
            )
            .into(),
          );
        }
        let stream = self.stream.unwrap();
        Ok(
          Sse::new(stream)
            .keep_alive(
              KeepAlive::new()
                .interval(Duration::from_secs(30))
                .text("keep-alive"),
            )
            .into_response(),
        )
      }
      None => Ok(
        response_builder.body(
          HyperBody::empty()
            .map_err(|e| axum::Error::new(e))
            .boxed_unsync(),
        )?,
      ),
    }
  }
}

pub struct StreamResponseWriter(pub RefCell<mpsc::Sender<Result<Event>>>);

impl Resource for StreamResponseWriter {
  fn name(&self) -> Cow<str> {
    "streamResponseWriter".into()
  }

  fn close(self: Rc<Self>) {
    drop(self.0.try_borrow_mut());
    drop(self);
  }
}
