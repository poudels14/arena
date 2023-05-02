use super::events::ServerEvent;
use super::start::WorkspaceServer;
use crate::server::events::ServerStarted;
use anyhow::Result;
use axum::{
  body::boxed,
  extract::State,
  http::Request,
  response::{IntoResponse, Response},
  routing::{self},
  Router,
};
use deno_core::{ByteString, ZeroCopyBuf};
use http::{header::HeaderName, HeaderMap, HeaderValue, Method, StatusCode};
use hyper::{body::HttpBody, Body};
use serde::Serialize;
use std::net::{Ipv4Addr, SocketAddr};
use std::str::FromStr;
use tokio::sync::mpsc;
use tower::util::ServiceExt;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::services::ServeDir;
use tracing::error;

// TODO(sagar): use fast serialization?
#[derive(Debug, Serialize)]
pub struct HttpRequest {
  pub url: String,

  pub method: String,

  pub headers: Vec<(String, String)>,

  pub body: Option<ZeroCopyBuf>,
}

#[derive(Debug, Serialize)]
pub struct HttpResponse {
  pub status: u16,

  pub headers: Vec<(ByteString, ByteString)>,

  // TODO(sagar): use bytes::Bytes instead
  pub body: Option<String>,

  /// set to true if the connection should be kept alive
  // TODO(sagar): find better way to close mpsc channel instead
  // of using this flag
  pub close: bool,
}

/// Create and listen to TCP socket for new requests
pub(crate) async fn listen(server: WorkspaceServer) -> Result<()> {
  let addr =
    SocketAddr::from((Ipv4Addr::from_str(&server.address)?, server.port));

  let cors = CorsLayer::new()
    .allow_methods([Method::GET])
    .allow_origin(AllowOrigin::list(vec![]));

  // Note(sagar): axum seems 25% slower than hyper but using axum for now
  // to make it easier to implement routing.
  // TODO(sagar): figure out why axum is slower than Deno's http benchmark
  // server running in JS
  let router = Router::new()
    .layer(cors)
    .route("/", routing::get(handle_request))
    .route("/*path", routing::get(handle_request))
    .route("/", routing::post(handle_request))
    .route("/*path", routing::post(handle_request))
    .with_state(server.clone());
  let app = axum::Server::bind(&addr);

  server.events.sender.send((
    ServerEvent::Started,
    serde_json::to_value(ServerStarted {
      address: addr.ip().to_string(),
      port: addr.port(),
    })?,
  ))?;

  app
    .serve(router.into_make_service())
    .with_graceful_shutdown(
      #[allow(unused_must_use)]
      async {
        // TODO(sagar): not sure if this is actually needed
        // Note(sagar): server terminated event can be sent from
        // else where, like JS runtime. so, if that event is
        // received, terinate the server
        server.events.wait_until(ServerEvent::Terminated).await;
        println!("Shutting down HTTP server...");
      },
    )
    .await
    .unwrap();

  Ok(())
}

async fn handle_request(
  State(server): State<WorkspaceServer>,
  mut req: Request<Body>,
) -> Response {
  // Note(sagar): serve static files directly from axum when not in dev mode
  if !server.dev_mode && req.uri().path().starts_with("/static") {
    let res = ServeDir::new(server.workspace.dir).oneshot(req).await;
    return match res {
      Ok(r) => r.map(boxed),
      Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    };
  }

  let body = req.body_mut().data().await;
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
      .collect(),
    body: body
      .and_then(|r| r.ok())
      .map(|r| r.to_vec().into_boxed_slice().into()),
  };

  let (tx, mut rx) = mpsc::channel(5);
  server
    .vm_serice
    .unwrap()
    .sender
    .send((request, tx))
    .await
    .unwrap();

  while let Some(r) = rx.recv().await {
    if r.close {
      let s = || {
        let mut headers = HeaderMap::new();
        for header in r.headers {
          headers.append(
            HeaderName::from_bytes(&header.0)?,
            HeaderValue::from_bytes(&header.1)?,
          );
        }

        <Result<Response>>::Ok(
          (
            StatusCode::from_u16(r.status)?,
            headers,
            r.body.unwrap_or("".to_owned()),
          )
            .into_response(),
        )
      };
      match s() {
        Ok(v) => return v,
        Err(e) => {
          error!("{:?}", e);
          return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
      }
    }
    error!("Response streaming not implemented!");
    return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
  }
  (StatusCode::INTERNAL_SERVER_ERROR).into_response()
}
