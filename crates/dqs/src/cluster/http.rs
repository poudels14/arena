use super::DqsCluster;
use anyhow::{anyhow, bail, Context, Result};
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::{routing, Router};
use axum_extra::extract::cookie::Cookie;
use cloud::acl::{Access, AclEntity};
use common::deno::extensions::server::response::HttpResponse;
use common::deno::extensions::server::{errors, HttpRequest};
use deno_core::ZeroCopyBuf;
use http::{Method, Request};
use hyper::body::HttpBody;
use hyper::Body;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::sync::mpsc;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::debug;

pub(crate) async fn start_server(
  cluster: DqsCluster,
  address: String,
  port: u16,
) -> Result<()> {
  let cors = CorsLayer::new()
    .allow_methods([Method::GET])
    .allow_origin(AllowOrigin::list(vec![]));

  let app = Router::new()
    .layer(cors)
    .layer(CompressionLayer::new())
    .route(
      "/api/query/:appId/:widgetId/:field",
      routing::get(handle_dqs_get_query),
    )
    .route(
      "/api/query/:appId/:widgetId/:field",
      routing::post(handle_dqs_mutate_query),
    )
    .with_state(cluster);

  let addr: SocketAddr = (Ipv4Addr::from_str(&address)?, port).into();
  debug!("DQS cluster started");
  axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await
    .unwrap();

  Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataQuerySearchParams {
  pub props: Option<String>,
  pub updated_at: Option<String>,
}

pub async fn handle_dqs_get_query(
  Path((app_id, widget_id, field)): Path<(String, String, String)>,
  Query(search_params): Query<DataQuerySearchParams>,
  State(cluster): State<DqsCluster>,
  req: Request<Body>,
) -> impl IntoResponse {
  pipe_dqs_request(
    &cluster,
    "QUERY",
    &app_id,
    &widget_id,
    &field,
    search_params,
    req,
  )
  .await
}

pub async fn handle_dqs_mutate_query(
  Path((app_id, widget_id, field)): Path<(String, String, String)>,
  Query(search_params): Query<DataQuerySearchParams>,
  State(cluster): State<DqsCluster>,
  req: Request<Body>,
) -> impl IntoResponse {
  pipe_dqs_request(
    &cluster,
    "MUTATION",
    &app_id,
    &widget_id,
    &field,
    search_params,
    req,
  )
  .await
}

pub async fn pipe_dqs_request(
  cluster: &DqsCluster,
  trigger: &str, // "QUERY" | "MUTATION"
  app_id: &str,
  widget_id: &str,
  field: &str,
  params: DataQuerySearchParams,
  mut req: Request<Body>,
) -> Result<HttpResponse, errors::Error> {
  let cookies = parse_cookies(&req);
  let user_id = cookies
    .get("user")
    .and_then(|u| get_user_id_from_cookie(u).ok())
    .unwrap_or("public".to_owned());

  let workspace_id = cluster
    .cache
    .get_workspace_id(app_id)
    .await
    .ok_or(errors::Error::NotFound)?;

  let acls = cluster
    .cache
    .get_workspace_acls(&workspace_id)
    .await
    .unwrap_or_default();

  let has_access = cloud::acl::has_entity_access(
    &acls,
    &user_id,
    Access::CanQuery,
    &workspace_id,
    AclEntity::App {
      id: app_id.to_string(),
      path: None,
    },
  )
  .map_err(|_| errors::Error::NotFound)?;

  if !has_access {
    return Err(errors::Error::NotFound);
  }

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

  let props = params
    .props
    .map(|p| serde_json::from_str(&p))
    .unwrap_or(Ok(json!({})))
    .context("failed to parse props")?;
  let request = HttpRequest {
    method: "POST".to_owned(),
    url: "http://0.0.0.0/execWidgetQuery".to_owned(),
    headers: vec![(("content-type".to_owned(), "application/json".to_owned()))],
    body: Some(ZeroCopyBuf::ToV8(Some(
      json!({
        "trigger": trigger,
        "workspaceId": "workspaceId",
        "appId": app_id,
        "widgetId": widget_id,
        "field": field,
        "props": props,
        "updatedAt": params.updated_at,
        "body": body,
      })
      .to_string()
      .as_bytes()
      .into(),
    ))),
  };

  let dqs_server = cluster.get_or_spawn_workspace_server(&workspace_id).await?;
  dqs_server
    .http_channel
    .send((request, tx))
    .await
    .map_err(|_| errors::Error::ResponseBuilder)?;

  rx.recv().await.ok_or(errors::Error::ResponseBuilder)
}

fn parse_cookies(req: &Request<Body>) -> HashMap<String, String> {
  Cookie::split_parse(
    req
      .headers()
      .get("cookie")
      .and_then(|c| c.to_str().ok())
      .unwrap_or_default(),
  )
  .into_iter()
  .fold(HashMap::new(), |mut map, c| {
    if let Ok(cookie) = c {
      map.insert(cookie.name().to_string(), cookie.value().to_string());
    }
    map
  })
}

fn get_user_id_from_cookie(token: &str) -> Result<String> {
  if let Ok(secret) = env::var("JWT_SIGNINIG_SECRET") {
    return jsonwebtoken::decode::<Value>(
      &token,
      &DecodingKey::from_secret(secret.as_ref()),
      &Validation::new(Algorithm::HS256),
    )
    .map(|r| {
      r.claims
        .get("data")
        .and_then(|data| data.get("id"))
        .and_then(|id| id.as_str().and_then(|v| Some(v.to_owned())))
        .ok_or(anyhow!("Error getting user_id from cookie"))
    })
    .context("JWT verification error")?;
  }
  bail!("JWT signing key not found");
}
