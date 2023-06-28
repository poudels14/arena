use super::errors::{self};
use super::response::ParsedHttpResponse;
use super::websocket::{self};
use anyhow::Result;
use axum::response::Response;
use deno_core::ZeroCopyBuf;
use http::{Method, Request};
use hyper::body::HttpBody;
use hyper::Body;
use serde::Serialize;
use std::path::PathBuf;
use tokio::sync::{mpsc, oneshot};

// TODO(sagar): use fast serialization?
#[derive(Debug, Serialize)]
pub struct HttpRequest {
  pub url: String,
  pub method: String,
  pub headers: Vec<(String, String)>,
  pub body: Option<ZeroCopyBuf>,
}

#[derive(Clone, Default)]
pub struct HandleOptions {
  // Serves static files from this path if set
  pub serve_dir: Option<PathBuf>,
}

/// Sends the request to the given http_channel and returns the
/// response returned by the channel
pub async fn pipe_request<'a>(
  http_channel: mpsc::Sender<(
    HttpRequest,
    oneshot::Sender<ParsedHttpResponse>,
  )>,
  mut req: Request<Body>,
) -> Result<Response, errors::Error> {
  let (tx, rx) = oneshot::channel::<ParsedHttpResponse>();

  let body = {
    match *req.method() {
      // Note(sagar): Deno's Request doesn't support body in GET/HEAD
      Method::GET | Method::HEAD => None,
      _ => {
        let b = req.body_mut().data().await;
        b.and_then(|r| r.ok()).map(|r| {
          <Box<[u8]> as Into<ZeroCopyBuf>>::into(r.to_vec().into_boxed_slice())
        })
      }
    }
  };

  let request = HttpRequest {
    method: req.method().as_str().to_string(),
    url: format!("http://0.0.0.0{}", req.uri().to_string()),
    headers: req
      .headers()
      .iter()
      .map(|(key, value)| {
        (
          key.to_string(),
          simdutf8::basic::from_utf8(value.as_bytes())
            .unwrap()
            .to_owned(),
        )
      })
      .collect::<Vec<(String, String)>>(),
    body,
  };

  http_channel
    .send((request, tx))
    .await
    .map_err(|_| errors::Error::ResponseBuilder)?;

  match rx.await {
    Ok(res) => {
      if res.has_upgrade_header() {
        return websocket::upgrade_to_websocket(req, res);
      }
      res.into_response().await
    }
    Err(_) => Err(errors::Error::ResponseBuilder),
  }
}

impl From<&str> for HttpRequest {
  fn from(body: &str) -> Self {
    HttpRequest {
      method: "GET".to_owned(),
      url: "http://0.0.0.0/".to_owned(),
      headers: vec![],
      body: Some(ZeroCopyBuf::ToV8(Some(
        body.to_owned().as_bytes().to_vec().into(),
      ))),
    }
  }
}

impl From<(Method, &str)> for HttpRequest {
  fn from((method, body): (Method, &str)) -> Self {
    HttpRequest {
      method: method.as_str().to_owned(),
      url: "http://0.0.0.0/".to_owned(),
      headers: vec![],
      body: Some(ZeroCopyBuf::ToV8(Some(
        body.to_owned().as_bytes().to_vec().into(),
      ))),
    }
  }
}
