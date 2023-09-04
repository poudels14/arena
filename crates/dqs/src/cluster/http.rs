use super::{DqsCluster, DqsServerOptions};
use crate::apps::App;
use crate::db;
use crate::db::app::apps;
use crate::server::entry::ServerEntry;
use anyhow::{anyhow, bail, Context, Result};
use axum::extract::{Path, Query, State};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::{routing, Router};
use axum_extra::extract::cookie::Cookie;
use cloud::acl::{Access, AclEntity};
use colored::Colorize;
use common::axum::logger;
use common::deno::extensions::server::request::read_htt_body_to_buffer;
use common::deno::extensions::server::response::ParsedHttpResponse;
use common::deno::extensions::server::{errors, HttpRequest};
use deno_core::{normalize_path, ZeroCopyBuf};
use diesel::prelude::*;
use http::{Method, Request};
use hyper::Body;
use indexmap::IndexMap;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::sync::oneshot;
use tower::ServiceBuilder;
use tower_http::compression::predicate::NotForContentType;
use tower_http::compression::{CompressionLayer, DefaultPredicate, Predicate};
use tower_http::cors::{AllowOrigin, CorsLayer};
use url::Url;

pub(crate) async fn start_server(
  cluster: DqsCluster,
  address: String,
  port: u16,
) -> Result<()> {
  let compression_predicate = DefaultPredicate::new()
    .and(NotForContentType::new(mime::TEXT_EVENT_STREAM.as_ref()));

  let app = Router::new()
    .route(
      "/w/:appId/widgets/:widgetId/api/:field",
      routing::get(handle_widget_get_query),
    )
    .route(
      "/w/:appId/widgets/:widgetId/api/:field",
      routing::post(handle_widgets_mutate_query),
    )
    .route("/w/:appId/", routing::get(handle_app_routes_index))
    .route("/w/:appId/*path", routing::get(handle_app_routes))
    .route("/w/:appId/", routing::post(handle_app_routes_index))
    .route("/w/:appId/*path", routing::post(handle_app_routes))
    .layer(
      ServiceBuilder::new()
        .layer(middleware::from_fn(logger::middleware))
        .layer(CompressionLayer::new().compress_when(compression_predicate))
        .layer(
          CorsLayer::new()
            .allow_methods([Method::GET])
            .allow_origin(AllowOrigin::list(vec![])),
        ),
    )
    .with_state(cluster);

  let addr: SocketAddr = (Ipv4Addr::from_str(&address)?, port).into();

  println!("{}", "Starting DQS cluster...".yellow().bold());
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

pub async fn handle_widget_get_query(
  Path((app_id, widget_id, field)): Path<(String, String, String)>,
  Query(search_params): Query<DataQuerySearchParams>,
  State(cluster): State<DqsCluster>,
  req: Request<Body>,
) -> impl IntoResponse {
  pipe_widget_query_request(
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

pub async fn handle_widgets_mutate_query(
  Path((app_id, widget_id, field)): Path<(String, String, String)>,
  Query(search_params): Query<DataQuerySearchParams>,
  State(cluster): State<DqsCluster>,
  req: Request<Body>,
) -> impl IntoResponse {
  pipe_widget_query_request(
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

pub async fn handle_app_routes_index(
  Path(app_id): Path<String>,
  Query(search_params): Query<IndexMap<String, String>>,
  State(cluster): State<DqsCluster>,
  req: Request<Body>,
) -> impl IntoResponse {
  pipe_app_request(app_id, "/".to_owned(), search_params, cluster, req).await
}

pub async fn handle_app_routes(
  Path((app_id, path)): Path<(String, String)>,
  Query(search_params): Query<IndexMap<String, String>>,
  State(cluster): State<DqsCluster>,
  req: Request<Body>,
) -> impl IntoResponse {
  pipe_app_request(app_id, path, search_params, cluster, req).await
}

pub async fn pipe_app_request(
  app_id: String,
  path: String,
  search_params: IndexMap<String, String>,
  cluster: DqsCluster,
  mut req: Request<Body>,
) -> Result<Response, errors::Error> {
  let workspace_id = cluster
    .cache
    .get_workspace_id(&app_id)
    .await
    .map_err(|e| {
      tracing::error!("Error getting workspace id: {}", e);
      errors::Error::AnyhowError(e.into())
    })?
    .ok_or(errors::Error::NotFound)?;

  let connection = &mut cluster
    .db_pool
    .clone()
    .get()
    .map_err(|e| anyhow!("{}", e))?;

  let app = db::app::table
    .filter(apps::id.eq(app_id.to_string()))
    .filter(apps::archived_at.is_null())
    .first::<db::app::App>(connection)
    .map_err(|e| anyhow!("Failed to load app from db: {}", e))?;

  let app_root_path =
    normalize_path(cluster.data_dir.join(format!("./apps/{}", app_id)));
  let dqs_server = cluster
    .get_or_spawn_dqs_server(DqsServerOptions {
      id: format!("app/{}", app_id),
      workspace_id,
      entry: ServerEntry::AppServer,
      app: Some(App {
        id: app_id,
        template: app.template.unwrap().try_into()?,
        root: app_root_path.clone(),
        registry: cluster.options.registry.clone(),
      }),
      ..Default::default()
    })
    .await?;

  let url = {
    let mut url = Url::parse(&format!("http://0.0.0.0/")).unwrap();
    url.set_path(&path);
    {
      let mut params = url.query_pairs_mut();
      search_params.iter().for_each(|e| {
        params.append_pair(e.0, e.1);
      });
    }
    url
  };

  let body = read_htt_body_to_buffer(&mut req).await?;
  let request = HttpRequest {
    method: req.method().to_string(),
    url: url.as_str().to_owned(),
    // TODO(sagar): pass in headers
    // don't pass in headers like `Cookie` that might contain
    // user auth credentials
    headers: vec![],
    body,
  };

  let (tx, rx) = oneshot::channel::<ParsedHttpResponse>();
  dqs_server
    .http_channel
    .send((request, tx))
    .await
    .map_err(|_| errors::Error::ResponseBuilder)?;

  let res = rx.await.map_err(|_| errors::Error::ResponseBuilder)?;
  res.into_response().await
}

pub async fn pipe_widget_query_request(
  cluster: &DqsCluster,
  trigger: &str, // "QUERY" | "MUTATION"
  app_id: &str,
  widget_id: &str,
  field: &str,
  params: DataQuerySearchParams,
  mut req: Request<Body>,
) -> Result<Response, errors::Error> {
  let cookies = parse_cookies(&req);
  let user_id = cookies
    .get("user")
    .and_then(|u| get_user_id_from_cookie(u).ok())
    .unwrap_or("public".to_owned());

  let workspace_id = cluster
    .cache
    .get_workspace_id(app_id)
    .await
    .map_err(|e| {
      tracing::error!("Error getting workspace id: {}", e);
      errors::Error::AnyhowError(e.into())
    })?
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

  let (tx, rx) = oneshot::channel::<ParsedHttpResponse>();
  let body = read_htt_body_to_buffer(&mut req).await?;

  let props = params
    .props
    .map(|p| serde_json::from_str(&p))
    .unwrap_or(Ok(json!({})))
    .context("failed to parse props")?;

  let request = HttpRequest {
    method: "POST".to_owned(),
    url: format!("http://0.0.0.0/{app_id}/widgets/{widget_id}/api/{field}"),
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

  let dqs_server = cluster
    .get_or_spawn_dqs_server(DqsServerOptions {
      id: format!("workspace/{}", workspace_id),
      workspace_id,
      entry: ServerEntry::DqsServer,
      ..Default::default()
    })
    .await?;
  dqs_server
    .http_channel
    .send((request, tx))
    .await
    .map_err(|_| errors::Error::ResponseBuilder)?;

  let res = rx.await.map_err(|_| errors::Error::ResponseBuilder)?;
  res.into_response().await
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
