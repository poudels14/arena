use crate::registry::Registry;
use crate::template;
use anyhow::{anyhow, Result};
use axum::body::Body;
use axum::body::HttpBody;
use axum::extract::Query;
use axum::extract::{Path, State};
use axum::middleware;
use axum::response;
use axum::response::{IntoResponse, Response};
use axum::{routing, Router};
use bytes::Bytes;
use common::axum::logger;
use http::header::CONTENT_TYPE;
use http::{HeaderValue, Method, StatusCode};
use std::collections::HashMap;
use std::env;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::str::FromStr;
use tower::ServiceBuilder;
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
  let app = Router::new()
    .route(
      "/static/templates/apps/*uri",
      routing::get(get_app_template_client_bundle),
    )
    .route("/static/*uri", routing::get(get_client_bundle))
    .route(
      "/server/templates/apps/*uri",
      routing::get(get_app_template_server_bundle),
    )
    .layer(
      ServiceBuilder::new()
        .layer(middleware::from_fn(logger::middleware))
        .layer(CompressionLayer::new())
        .layer(
          CorsLayer::new()
            .allow_methods([Method::GET])
            .allow_origin(AllowOrigin::list(vec![])),
        ),
    )
    .with_state(AppState {
      registry: registry.clone(),
    });

  let addr: SocketAddr = (Ipv4Addr::from_str(&address)?, port).into();
  println!("Registry server started!");
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
  get_bundle_internal(&format!("static/{0}", uri), &state).await
}

async fn get_app_template_client_bundle(
  Path(uri): Path<String>,
  State(state): State<AppState>,
) -> response::Result<Response> {
  if let Ok(t) = template::parse(&uri) {
    return get_bundle_internal(
      &format!("static/templates/apps/{0}/{1}.js", t.id, t.version),
      &state,
    )
    .await;
  };

  Ok(((StatusCode::NOT_FOUND, "Not found")).into_response())
}

async fn get_app_template_server_bundle(
  Path(uri): Path<String>,
  Query(search_params): Query<HashMap<String, String>>,
  State(state): State<AppState>,
) -> response::Result<Response> {
  // Note(sagar): only allow access to server bundles if `env.API_KEY` matches
  // with query param `API_KEY`
  match search_params.get("API_KEY") {
    Some(api_key) if env::var("API_KEY").ok().eq(&Some(api_key.to_owned())) => {
      if let Ok(t) = template::parse(&uri) {
        return get_bundle_internal(
          &format!("server/templates/apps/{0}/{1}.js", t.id, t.version),
          &state,
        )
        .await;
      };
    }
    _ => {}
  }

  Ok(((StatusCode::NOT_FOUND, "Not found")).into_response())
}

async fn get_bundle_internal(
  uri: &str,
  state: &AppState,
) -> response::Result<Response> {
  let file = state.registry.get_contents(uri).await;

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
