use super::errors::{self, Error};
use super::websocket::WebsocketStream;
use bytes::Bytes;
use deno_core::{ByteString, StringOrBuffer};
use http::header::UPGRADE;
use http::{HeaderName, HeaderValue, Response};
use http_body::combinators::UnsyncBoxBody;
use hyper::body::HttpBody;
use hyper::Body;
use std::time::Instant;
use tokio::sync::oneshot;

pub type HttpResponse = Response<UnsyncBoxBody<Bytes, Error>>;

#[derive(Debug)]
pub struct ParsedHttpResponse {
  pub rid: u32,
  pub status: u16,
  pub headers: Vec<(ByteString, ByteString)>,
  pub data: Option<StringOrBuffer>,
  pub websocket_tx: Option<oneshot::Sender<WebsocketStream>>,
}

pub struct HttpResponseMetata {
  pub method: String,
  pub path: String,
  pub req_received_at: Instant,
}

impl ParsedHttpResponse {
  pub fn has_upgrade_header(&self) -> bool {
    self
      .headers
      .iter()
      .find(|(name, _)| {
        HeaderName::from_bytes(name)
          .map(|h| h == UPGRADE)
          .unwrap_or(false)
      })
      .is_some()
  }
}

impl Into<Result<HttpResponse, errors::Error>> for ParsedHttpResponse {
  fn into(self) -> Result<HttpResponse, errors::Error> {
    let mut response_builder = Response::builder().status(self.status);
    for header in &self.headers {
      response_builder = response_builder.header(
        HeaderName::from_bytes(&header.0)?,
        HeaderValue::from_bytes(&header.1)?,
      );
    }

    Ok(
      response_builder.body(
        Body::from(Bytes::from(
          self
            .data
            .unwrap_or(StringOrBuffer::String("".to_owned()))
            .to_vec(),
        ))
        .map_err(Into::into)
        .boxed_unsync(),
      )?,
    )
  }
}
