use crate::registry::Registry;
use crate::template;
use anyhow::anyhow;
use anyhow::Result;
use axum::body::Body;
use axum::body::HttpBody;
use axum::extract::{Path, State};
use axum::response;
use axum::response::{IntoResponse, Response};
use axum::{routing, Router};
use bytes::Bytes;
use http::header::CONTENT_TYPE;
use http::{HeaderValue, Method, StatusCode};
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::str::FromStr;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::debug;

#[derive(Clone)]
pub struct AppState {
  pub registry: Registry,
}

pub(crate) async fn start(
  address: String,
  port: u16,
  registry: Registry,
) -> Result<()> {
  let cors = CorsLayer::new()
    .allow_methods([Method::GET])
    .allow_origin(AllowOrigin::list(vec![]));

  let app = Router::new()
    .layer(cors)
    .layer(CompressionLayer::new())
    .route(
      "/static/templates/apps/*uri",
      routing::get(get_app_template_client_bundle),
    )
    .route("/static/*uri", routing::get(get_client_bundle))
    .route(
      "/server/templates/apps/*uri",
      routing::get(get_app_template_server_bundle),
    )
    .with_state(AppState {
      registry: registry.clone(),
    });

  let addr: SocketAddr = (Ipv4Addr::from_str(&address)?, port).into();
  println!("JS registry started");
  axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await
    .unwrap();

  Ok(())
}

async fn get_client_bundle(
  Path(uri): Path<String>,
  State(state): State<AppState>,
) -> response::Result<Response> {
  let file = state
    .registry
    .get_contents(&format!("static/{0}", uri))
    .await;

  match file {
    Ok(content) if content.is_some() => {
      let file = content.unwrap();
      return Ok(
        Response::builder()
          .header(
            CONTENT_TYPE,
            HeaderValue::from_str(file.mime.as_ref()).unwrap(),
          )
          .body(
            Body::from(Bytes::from(file.content))
              .map_err(|e| {
                debug!("{e}");
                axum::Error::new(anyhow!("Unexpected error"))
              })
              .boxed_unsync(),
          )
          .unwrap_or_default(),
      );
    }
    Err(e) => {
      debug!("{e}");
      return Ok(
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
          .into_response(),
      );
    }
    Ok(_) => {}
  };

  Ok(((StatusCode::NOT_FOUND, "Not found")).into_response())
}

async fn get_app_template_client_bundle(
  Path(uri): Path<String>,
  State(state): State<AppState>,
) -> response::Result<Response> {
  match template::parse(&uri) {
    Ok(t) => {
      let file = state
        .registry
        .get_contents(&format!(
          "static/templates/apps/{0}/{1}.js",
          t.id, t.version
        ))
        .await;

      match file {
        Ok(content) if content.is_some() => {
          let file = content.unwrap();
          return Ok(
            Response::builder()
              .header(
                CONTENT_TYPE,
                HeaderValue::from_str(file.mime.as_ref()).unwrap(),
              )
              .body(
                Body::from(Bytes::from(file.content))
                  .map_err(|e| {
                    debug!("{e}");
                    axum::Error::new(anyhow!("Unexpected error"))
                  })
                  .boxed_unsync(),
              )
              .unwrap_or_default(),
          );
        }
        Err(e) => {
          debug!("{e}");
          return Ok(
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
              .into_response(),
          );
        }
        Ok(_) => {}
      };
    }
    _ => {}
  };

  Ok(((StatusCode::NOT_FOUND, "Not found")).into_response())
}

async fn get_app_template_server_bundle(
  Path(_uri): Path<String>,
  State(_state): State<AppState>,
) -> Response {
  // TODO(sagar): use this to load server module from DQS
  StatusCode::NOT_FOUND.into_response()
}
