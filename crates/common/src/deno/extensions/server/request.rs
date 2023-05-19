use super::errors;
use super::resonse::HttpResponse;
use deno_core::ZeroCopyBuf;
use http::{Method, Request, Response};
use hyper::body::HttpBody;
use hyper::Body;
use serde::Serialize;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tower::ServiceExt;
use tower_http::services::ServeDir;

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

pub(super) async fn handle_request(
  req_sender: mpsc::Sender<(HttpRequest, mpsc::Sender<HttpResponse>)>,
  options: HandleOptions,
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

  match options.serve_dir {
    Some(base_dir) if req.uri().path().starts_with("/static") => {
      let res = ServeDir::new(base_dir).oneshot(req).await;
      return Ok(
        res.map(|r| r.map(|body| body.map_err(Into::into).boxed_unsync()))?,
      );
    }
    _ => {}
  }

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

  req_sender.send((request, tx)).await.unwrap();
  if let Some(res) = rx.recv().await {
    return Response::builder()
      .body(res.boxed_unsync())
      .map_err(Into::into);
  }

  errors::not_found()
}
