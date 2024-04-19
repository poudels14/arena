use std::borrow::Cow;
use std::cell::{Ref, RefCell};
use std::time::{Duration, Instant};

use anyhow::Result;
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use deno_core::{ByteString, Resource, StringOrBuffer};
use derivative::Derivative;
use http::header::{CONTENT_TYPE, UPGRADE};
use http::{HeaderName, HeaderValue};
use http_body::Body;
use hyper::Body as HyperBody;
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio_stream::wrappers::ReceiverStream;

use super::errors::{self};
use super::sse::{Event, KeepAlive, Sse};
use super::websocket::WebsocketStream;

pub trait IntoHttpResponse: IntoResponse {
  fn into_response(self) -> axum::response::Response;
}

#[derive(Debug, Default)]
pub struct ParsedHttpResponse {
  pub rid: u32,
  pub status: u16,
  pub headers: Vec<(ByteString, ByteString)>,
  pub data: Option<StringOrBuffer>,
  pub stream: Option<StreamResponseReader>,
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
        let is_event_stream = self
          .get_header(CONTENT_TYPE)
          .map(|h| h == mime::TEXT_EVENT_STREAM.as_ref().as_bytes())
          .unwrap_or(true);
        let stream = self.stream.unwrap();
        if is_event_stream {
          return Ok(
            Sse::new(stream.get_event_stream())
              .keep_alive(
                KeepAlive::new()
                  .interval(Duration::from_secs(30))
                  .text("keep-alive"),
              )
              .into_response(),
          );
        } else {
          return Ok(
            response_builder.body(
              HyperBody::wrap_stream(stream.get_bytes_stream())
                .map_err(|e| axum::Error::new(e))
                .boxed_unsync(),
            )?,
          );
        }
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

#[derive(Debug, Deserialize)]
pub enum StreamType {
  Bytes,
  Events,
}

pub fn channel(
  stream_type: StreamType,
) -> (StreamResponseWriter, StreamResponseReader) {
  match stream_type {
    StreamType::Bytes => {
      let (tx, rx) = mpsc::channel(100);
      (
        StreamResponseWriter::Bytes(tx.into()),
        StreamResponseReader::Bytes(rx.into()),
      )
    }
    StreamType::Events => {
      let (tx, rx) = mpsc::channel(100);
      (
        StreamResponseWriter::Events(tx.into()),
        StreamResponseReader::Events(rx.into()),
      )
    }
  }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub enum StreamResponseReader {
  // Used when streaming data
  Bytes(#[derivative(Debug = "ignore")] ReceiverStream<Result<Bytes>>),
  // Used for text/event-stream response type
  Events(#[derivative(Debug = "ignore")] ReceiverStream<Result<Event>>),
}

impl StreamResponseReader {
  pub fn get_bytes_stream(self) -> ReceiverStream<Result<Bytes>> {
    match self {
      Self::Bytes(stream) => stream,
      Self::Events(_) => {
        unreachable!("Can't get normal stream from event stream")
      }
    }
  }

  pub fn get_event_stream(self) -> ReceiverStream<Result<Event>> {
    match self {
      Self::Bytes(_) => {
        unreachable!("Can't get event stream from normal stream")
      }
      Self::Events(stream) => stream,
    }
  }
}

pub enum StreamResponseWriter {
  Bytes(RefCell<mpsc::Sender<Result<Bytes>>>),
  Events(RefCell<mpsc::Sender<Result<Event>>>),
}

impl StreamResponseWriter {
  #[inline]
  pub fn event_sender<'a>(&'a self) -> Ref<'a, mpsc::Sender<Result<Event>>> {
    match self {
      Self::Bytes(_) => unimplemented!(),
      Self::Events(tx) => tx.borrow(),
    }
  }

  #[inline]
  pub async fn write_bytes(&self, bytes: Result<Bytes>) -> Result<()> {
    match self {
      Self::Bytes(tx) => {
        tx.borrow().send(bytes).await?;
        Ok(())
      }
      Self::Events(_) => unimplemented!(),
    }
  }

  #[inline]
  pub async fn write_text(&self, text: &str) -> Result<()> {
    match self {
      Self::Bytes(_) => unimplemented!(),
      Self::Events(tx) => {
        tx.borrow()
          .send(Ok(Event::default().data::<&str>(&text)))
          .await?;
        Ok(())
      }
    }
  }

  #[inline]
  pub async fn write_event(&self, event: Event) -> Result<()> {
    match self {
      Self::Bytes(_) => unimplemented!(),
      Self::Events(tx) => {
        tx.borrow().send(Ok(event)).await?;
        Ok(())
      }
    }
  }
}

impl Resource for StreamResponseWriter {
  fn name(&self) -> Cow<str> {
    "StreamResponseWriter".into()
  }
}
