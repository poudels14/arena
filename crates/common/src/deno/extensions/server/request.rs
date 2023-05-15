use deno_core::ZeroCopyBuf;
use http::{Method, Request, Response, StatusCode};
use hyper::body::HttpBody;
use hyper::Body;
use serde::Serialize;
use tokio::sync::mpsc;

// TODO(sagar): use fast serialization?
#[derive(Debug, Serialize)]
pub struct HttpRequest {
  pub url: String,

  pub method: String,

  pub headers: Vec<(String, String)>,

  pub body: Option<ZeroCopyBuf>,
}

pub(super) async fn handle_request(
  req_sender: mpsc::Sender<(HttpRequest, mpsc::Sender<Response<Body>>)>,
  mut req: Request<Body>,
) -> Result<Response<Body>, http::Error> {
  let (tx, mut rx) = mpsc::channel::<Response<Body>>(10);

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

  req_sender.send((request, tx)).await.unwrap();
  if let Some(res) = rx.recv().await {
    return Ok(res);
  }

  Response::builder()
    .status(StatusCode::INTERNAL_SERVER_ERROR)
    .body(Body::from("Internal server error"))
}
