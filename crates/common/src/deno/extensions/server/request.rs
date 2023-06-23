use super::errors;
use super::response::HttpResponse;
use deno_core::ZeroCopyBuf;
use http::{Method, Request};
use hyper::body::HttpBody;
use hyper::Body;
use serde::Serialize;
use std::path::PathBuf;
use tokio::sync::mpsc;

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
pub async fn pipe_request(
  http_channel: mpsc::Sender<(HttpRequest, mpsc::Sender<HttpResponse>)>,
  mut req: Request<Body>,
) -> Result<HttpResponse, errors::Error> {
  let (tx, mut rx) = mpsc::channel::<HttpResponse>(10);

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
          String::from_utf8(value.as_bytes().to_owned()).unwrap(),
        )
      })
      .collect::<Vec<(String, String)>>(),
    body,
  };

  http_channel
    .send((request, tx))
    .await
    .map_err(|_| errors::Error::ResponseBuilder)?;

  rx.recv().await.ok_or(errors::Error::ResponseBuilder)
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
