use std::collections::HashMap;
use std::env;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use axum::extract::{Json, Path, Query, State};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::MethodFilter;
use axum::{routing, Router};
use axum_extra::extract::cookie::Cookie;
use cloud::acl::{Access, AclEntity};
use cloud::identity::Identity;
use cloud::pubsub::EventSink;
use cloud::pubsub::Subscriber;
use colored::Colorize;
use common::axum::logger;
use common::deno::extensions::server::request::read_http_body_to_buffer;
use common::deno::extensions::server::response::ParsedHttpResponse;
use common::deno::extensions::server::{errors, HttpRequest};
use deno_core::{normalize_path, ZeroCopyBuf};
use http::StatusCode;
use http::{Method, Request};
use hyper::Body;
use indexmap::IndexMap;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use nanoid::nanoid;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::compression::predicate::NotForContentType;
use tower_http::compression::{CompressionLayer, DefaultPredicate, Predicate};
use tower_http::cors::{AllowOrigin, CorsLayer};
use url::Url;
use uuid::Uuid;

use super::{DqsCluster, DqsServerOptions};
use crate::arena::App;
use crate::arena::MainModule;
use crate::arena::Template;

pub(crate) async fn start_server(
  cluster: DqsCluster,
  address: String,
  port: u16,
) -> Result<()> {
  let compression_predicate = DefaultPredicate::new()
    .and(NotForContentType::new(mime::TEXT_EVENT_STREAM.as_ref()));

  let app = Router::new()
    .route(
      "/w/plugin/:pluginNamespace/:pluginId/:version/:workflowId/*path",
      routing::post(pipe_execute_plugin_workflow_request),
    )
    .route(
      "/w/apps/:appId/widgets/:widgetId/api/:field",
      routing::get(handle_widget_get_query),
    )
    .route(
      "/w/apps/:appId/widgets/:widgetId/api/:field",
      routing::post(handle_widgets_mutate_query),
    )
    .route(
      "/w/apps/:appId/",
      routing::on(MethodFilter::all(), handle_app_routes_index),
    )
    .route(
      "/w/apps/:appId/*path",
      routing::on(MethodFilter::all(), handle_app_routes),
    )
    .route(
      "/_admin/healthy",
      routing::get(|| async { (StatusCode::OK, "Ok") }),
    )
    // TODO: listen to pub/sub in different port
    // Subscribe to all pub/sub events for all apps/workflows in the given
    // workspace that are running on this dqs server
    .route(
      "/w/subscribe/:workspaceId/*path",
      routing::get(subscribe_to_events),
    )
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

  // TODO(sagar): listen on multiple ports so that a lot more traffic
  // can be served from single cluster
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
  pipe_app_request(cluster, app_id, "/".to_owned(), search_params, req).await
}

pub async fn handle_app_routes(
  Path((app_id, path)): Path<(String, String)>,
  Query(search_params): Query<IndexMap<String, String>>,
  State(cluster): State<DqsCluster>,
  req: Request<Body>,
) -> impl IntoResponse {
  pipe_app_request(cluster, app_id, path, search_params, req).await
}

pub async fn pipe_app_request(
  cluster: DqsCluster,
  app_id: String,
  path: String,
  search_params: IndexMap<String, String>,
  mut req: Request<Body>,
) -> Result<Response, errors::Error> {
  let (_, app) =
    authenticate_user(&cluster, &app_id, Some(path.clone()), &req).await?;

  let app_root_path =
    normalize_path(cluster.data_dir.join(format!("./apps/{}", app_id)));
  let dqs_server = cluster
    .get_or_spawn_dqs_server(DqsServerOptions {
      id: format!("app/{}", app_id),
      workspace_id: app.workspace_id.clone(),
      root: Some(app_root_path),
      module: MainModule::App { app },
      db_pool: cluster.db_pool.clone(),
      dqs_egress_addr: cluster.options.dqs_egress_addr,
      registry: cluster.options.registry.clone(),
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

  let body = read_http_body_to_buffer(&mut req).await?;
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
  let path = format!("/{app_id}/widgets/{widget_id}/api/{field}");
  let (_, app) =
    authenticate_user(cluster, &app_id, Some(path.clone()), &req).await?;

  let (tx, rx) = oneshot::channel::<ParsedHttpResponse>();
  let body = read_http_body_to_buffer(&mut req).await?;

  let props = params
    .props
    .map(|p| serde_json::from_str(&p))
    .unwrap_or(Ok(json!({})))
    .context("failed to parse props")?;

  let request = HttpRequest {
    method: "POST".to_owned(),
    url: format!("http://0.0.0.0{path}"),
    // TODO(sagar): maybe send some headers?
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

  let workspace_id = app.workspace_id;
  let dqs_server = cluster
    .get_or_spawn_dqs_server(DqsServerOptions {
      id: format!("workspace/{}", workspace_id),
      workspace_id,
      root: None,
      module: MainModule::WidgetQuery,
      db_pool: cluster.db_pool.clone(),
      dqs_egress_addr: cluster.options.dqs_egress_addr,
      registry: cluster.options.registry.clone(),
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

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutePluginQuery {
  workspace_id: String,
  input: Value,
}

pub async fn pipe_execute_plugin_workflow_request(
  Path((plugin_namespace, plugin_id, version, workflow_id, path)): Path<(
    String,
    String,
    String,
    String,
    String,
  )>,
  State(cluster): State<DqsCluster>,
  Json(body): Json<ExecutePluginQuery>,
) -> Result<Response, errors::Error> {
  let plugin_id = format!("{}/{}", plugin_namespace, plugin_id);

  let (dqs_server, _) = cluster
    .spawn_dqs_server(DqsServerOptions {
      id: format!(
        "plugin/{}/{}/workflow/{}/{}",
        plugin_id,
        version,
        workflow_id,
        // Note(sagar): use a random id since this is a one-off dqs runtime
        // and the runtime isn't reused
        nanoid!(8, &nanoid::alphabet::SAFE)
      ),
      workspace_id: body.workspace_id,
      root: None,
      module: MainModule::Workflow {
        // TODO(sagar): use unique workflow id
        id: workflow_id.clone(),
        name: workflow_id.clone(),
        plugin: Template {
          id: plugin_id.clone(),
          version: version.clone(),
        },
      },
      db_pool: cluster.db_pool.clone(),
      dqs_egress_addr: cluster.options.dqs_egress_addr,
      registry: cluster.options.registry.clone(),
    })
    .await?;

  let request = HttpRequest {
    method: "POST".to_owned(),
    url: format!("http://0.0.0.0/{path}",),
    headers: vec![],
    body: Some(ZeroCopyBuf::ToV8(Some(
      serde_json::to_vec(&json!({
            "workflowId": workflow_id,
            "input": body.input
      }))
      .context("Error parsing workflow input")?
      .into_boxed_slice(),
    ))),
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

async fn subscribe_to_events(
  State(cluster): State<DqsCluster>,
  Path((workspace_id, _path)): Path<(String, String)>,
  mut req: Request<Body>,
) -> Result<http::Response<hyper::Body>, errors::Error> {
  let cookies = parse_cookies(&req);

  // TODO(sagar): authenticate the non-user subscriber like apps and
  // set proper node info
  let identity =
    get_identity_from_cookie(&cookies).unwrap_or(Identity::Unknown);

  let (response, upgrade_fut) = fastwebsockets::upgrade::upgrade(&mut req)
    .map_err(|e| anyhow!("Error upgrading websocket connection: {}", e))?;

  tokio::spawn(async move {
    let ws = upgrade_fut.await;

    match ws {
      Ok(ws) => {
        if let Ok(exchange) = cluster.get_exchange(&workspace_id).await {
          let ws = fastwebsockets::FragmentCollector::new(ws);
          let res = exchange
            .add_subscriber(Subscriber {
              id: Uuid::new_v4().to_string(),
              identity,
              out_stream: EventSink::Websocket(Arc::new(Mutex::new(ws))),
              // TODO(sagar): set event filter based on the path
              filter: Default::default(),
            })
            .await;
          if res.is_err() {
            tracing::error!(
              "Error sending event to a subscriber [workspace_id = {}]: {:?}",
              workspace_id,
              res
            );
          }
        }
      }
      Err(e) => {
        tracing::error!("Error upgrading websocket connection: {}", e)
      }
    }
  });

  Ok::<_, errors::Error>(response.into())
}

/// Returns tuple of (identity, App) if authorization succeeds
async fn authenticate_user(
  cluster: &DqsCluster,
  app_id: &str,
  path: Option<String>,
  req: &Request<Body>,
) -> Result<(Identity, App), errors::Error> {
  let cookies = parse_cookies(&req);
  let identity =
    get_identity_from_cookie(&cookies).unwrap_or(Identity::Unknown);

  let app = cluster
    .cache
    .get_app(app_id)
    .await
    .map_err(|e| {
      tracing::error!("Error getting workspace id: {}", e);
      errors::Error::AnyhowError(e.into())
    })?
    .ok_or(errors::Error::NotFound)?;

  let acls = cluster
    .cache
    .get_workspace_acls(&app.workspace_id)
    .await
    .unwrap_or_default();

  let has_access = cloud::acl::has_entity_access(
    &acls,
    &identity,
    Access::CanQuery,
    &app.workspace_id,
    AclEntity::App {
      id: app.id.to_string(),
      path,
    },
  )
  .map_err(|_| errors::Error::NotFound)?;

  if !has_access {
    return Err(errors::Error::NotFound);
  }
  Ok((identity, app))
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

fn get_identity_from_cookie(
  cookies: &HashMap<String, String>,
) -> Result<Identity> {
  let token = cookies
    .get("user")
    .ok_or(anyhow!("User not found in cookie"))?;

  let secret = env::var("JWT_SIGNINIG_SECRET")?;
  jsonwebtoken::decode::<Value>(
    token,
    &DecodingKey::from_secret(secret.as_ref()),
    &Validation::new(Algorithm::HS256),
  )
  .context("JWT verification error")
  .and_then(|mut r| {
    let claims = r
      .claims
      .as_object_mut()
      .ok_or(anyhow!("Invalid JWT token"))?;

    // when deserializing enum, can't have unspecified fields
    claims.retain(|k, _| k == "user" || k == "app" || k == "workflow");

    serde_json::from_value(r.claims)
      .context("Failed to parse identity from cookie")
  })
}
