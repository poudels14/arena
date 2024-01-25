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
use axum::RequestExt;
use axum::{routing, Router};
use axum_extra::extract::cookie::Cookie;
use cloud::acl::{Access, AclEntity};
use cloud::identity::Identity;
use cloud::pubsub::EventSink;
use cloud::pubsub::Subscriber;
use colored::Colorize;
use common::axum::logger;
use diesel::prelude::*;
use http::StatusCode;
use http::{Method, Request};
use hyper::Body;
use indexmap::IndexMap;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use runtime::extensions::server::errors::Error;
use runtime::extensions::server::request::read_http_body_to_buffer;
use runtime::extensions::server::response::ParsedHttpResponse;
use runtime::extensions::server::{errors, HttpRequest};
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
use crate::arena::workflow::PluginWorkflow;
use crate::arena::workflow::WorkflowTemplate;
use crate::arena::App;
use crate::arena::MainModule;
use crate::cluster::server::DqsServerStatus;
use crate::db::workflow::workflow_runs;
use crate::db::workflow::WorkflowRun;
use crate::runtime::Command;

impl DqsCluster {
  pub(crate) async fn start_server(
    &self,
    shutdown_signal: oneshot::Receiver<()>,
  ) -> Result<()> {
    let compression_predicate = DefaultPredicate::new()
      .and(NotForContentType::new(mime::TEXT_EVENT_STREAM.as_ref()));

    let app = Router::new()
      .route(
        "/w/workflow/:workflowId/*path",
        routing::on(MethodFilter::all(), pipe_plugin_workflow_request),
      )
      // .route(
      //   "/w/apps/:appId/widgets/:widgetId/api/:field",
      //   routing::get(handle_widget_get_query),
      // )
      // .route(
      //   "/w/apps/:appId/widgets/:widgetId/api/:field",
      //   routing::post(handle_widgets_mutate_query),
      // )
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
      // TODO(sagar): change the path to allow only subscribing to a single
      // app in general. But, when a notification type message is published,
      // send that to all subscribers of all apps of that workspace.
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
      .with_state(self.clone());

    // TODO(sagar): listen on multiple ports so that a lot more traffic
    // can be served from single cluster
    let addr: SocketAddr = (
      Ipv4Addr::from_str(&self.options.address)?,
      self.options.port,
    )
      .into();

    println!(
      "{}",
      format!("Starting DQS cluster on port {}...", self.options.port)
        .yellow()
        .bold()
    );
    self.mark_node_as_online()?;
    axum::Server::bind(&addr)
      .serve(app.into_make_service())
      .with_graceful_shutdown(async {
        shutdown_signal.await.ok();
        let _ = self.mark_node_as_terminating();
      })
      .await?;

    self.mark_node_as_terminated()?;

    // Terminate all server threads
    for server in self.servers.iter_mut() {
      match server.value() {
        DqsServerStatus::Ready(server) => {
          let commands_channel = server.commands_channel.clone();
          let _ = commands_channel.send(Command::Terminate).await;
          let _ = server.thread_handle.lock().unwrap().take().unwrap().join();
        }
        _ => {}
      };
    }

    Ok(())
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataQuerySearchParams {
  pub props: Option<String>,
  pub updated_at: Option<String>,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[tracing::instrument(skip_all, err, level = "trace")]
pub async fn pipe_app_request(
  cluster: DqsCluster,
  app_id: String,
  path: String,
  search_params: IndexMap<String, String>,
  mut req: Request<Body>,
) -> Result<Response, errors::Error> {
  let (_, app) =
    authenticate_user(&cluster, &app_id, Some(path.clone()), &req).await?;

  let dqs_server = cluster
    .get_or_spawn_dqs_server(DqsServerOptions {
      id: format!("app/{}", app_id),
      version: app.template.version.clone(),
      workspace_id: app.workspace_id.clone(),
      root: None,
      module: MainModule::App { app },
      db_pool: cluster.db_pool.clone(),
      dqs_egress_addr: cluster.options.dqs_egress_addr,
      registry: cluster.options.registry.clone(),
    })
    .await
    .map_err(|_| errors::Error::ServiceUnavailable)?;

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
    .map_err(|_| errors::Error::ServiceUnavailable)?;

  let res = rx.await.map_err(|_| errors::Error::ResponseBuilder)?;
  res.into_response().await
}

#[tracing::instrument(skip_all, err, level = "trace")]
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
    body: Some(
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
      .into_bytes()
      .into(),
    ),
  };

  let workspace_id = app.workspace_id;
  let dqs_server = cluster
    .get_or_spawn_dqs_server(DqsServerOptions {
      id: format!("workspace/{}", workspace_id),
      version: app.template.version.clone(),
      workspace_id,
      root: None,
      module: MainModule::WidgetQuery,
      db_pool: cluster.db_pool.clone(),
      dqs_egress_addr: cluster.options.dqs_egress_addr,
      registry: cluster.options.registry.clone(),
    })
    .await
    .map_err(|_| errors::Error::ServiceUnavailable)?;

  dqs_server
    .http_channel
    .send((request, tx))
    .await
    .map_err(|_| errors::Error::ServiceUnavailable)?;

  let res = rx.await.map_err(|_| errors::Error::ResponseBuilder)?;
  res.into_response().await
}

pub async fn pipe_plugin_workflow_request(
  Path((workflow_id, path)): Path<(String, String)>,
  State(cluster): State<DqsCluster>,
  req: Request<Body>,
) -> Result<Response, errors::Error> {
  let identity =
    parse_identity_from_header(&req).map_err(|_| errors::Error::NotFound)?;

  let mut connection = cluster
    .db_pool
    .get()
    .map_err(|_| anyhow!("Database connection error"))?;

  let wf_run: WorkflowRun = workflow_runs::table
    .filter(workflow_runs::id.eq(&workflow_id))
    .first::<WorkflowRun>(&mut connection)
    .optional()
    .map_err(|e| anyhow!("Failed to load Workflow run from db: {}", e))?
    .ok_or(Error::NotFound)?;

  let Json(body): Json<Value> = req
    .extract()
    .await
    .map_err(|_| Error::RequestParsingError)?;

  let template = serde_json::from_value::<WorkflowTemplate>(wf_run.template)
    .map_err(|_| Error::BadRequest(Some("Invalid workflow tempalte")))?;

  let can_access = match identity {
    // Only allow the app that created the workflow to access it for now
    Identity::App {
      id,
      system_originated,
    } => {
      system_originated.unwrap_or(false)
        && wf_run
          .parent_app_id
          .map(|parent_app_id| parent_app_id == id)
          .unwrap_or(false)
    }
    _ => false,
  };
  if !can_access {
    return Err(Error::NotFound);
  }

  let (workflow, module) = match template {
    WorkflowTemplate::Plugin { plugin, slug } => {
      let workflow = PluginWorkflow {
        id: workflow_id.clone(),
        plugin,
        slug,
      };

      (workflow.clone(), MainModule::PluginWorkflowRun { workflow })
    }
    #[allow(unreachable_patterns)]
    _ => {
      return Err(Error::BadRequest(Some("Only plugin workflow supported")));
    }
  };

  let (dqs_server, _) = cluster
    .spawn_dqs_server(DqsServerOptions {
      id: format!("workflow/{}", workflow_id),
      version: "0.0.0".to_owned(),
      workspace_id: wf_run.workspace_id,
      root: None,
      module,
      db_pool: cluster.db_pool.clone(),
      dqs_egress_addr: cluster.options.dqs_egress_addr,
      registry: cluster.options.registry.clone(),
    })
    .await?;

  let request = HttpRequest {
    method: "POST".to_owned(),
    url: format!("http://0.0.0.0/{path}",),
    headers: vec![],
    body: Some(
      serde_json::to_vec(&json!({
        "workflow": workflow,
        "input": body
      }))
      .context("Error serializing workflow request body")?
      .into(),
    ),
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
  let identity = parse_identity_from_header(&req).unwrap_or(Identity::Unknown);

  // Disable events subscription for unknown user
  if identity == Identity::Unknown {
    return Err(errors::Error::NotFound);
  }

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
  let identity = parse_identity_from_header(req).unwrap_or(Identity::Unknown);
  tracing::trace!("identity = {:?}", identity);

  let app = cluster
    .cache
    .get_app(app_id)
    .await
    .map_err(|e| {
      tracing::error!("Error getting workspace id: {}", e);
      errors::Error::AnyhowError(e.to_string())
    })?
    .ok_or(errors::Error::NotFound)?;

  let acls = cluster
    .cache
    .get_workspace_acls(&app.workspace_id)
    .await
    .unwrap_or_default();
  tracing::trace!("acls = {:?}", acls);

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

fn parse_identity_from_header(req: &Request<Body>) -> Result<Identity> {
  let cookies = { parse_cookies(req) };
  let token = cookies.get("user").map(|v| v.as_str()).or_else(|| {
    req
      .headers()
      .get("x-portal-authentication")
      .and_then(|c| c.to_str().ok())
  });

  if token.is_none() {
    return Ok(Identity::Unknown);
  }
  let token = token.unwrap();

  let secret = env::var("JWT_SIGNING_SECRET")?;
  jsonwebtoken::decode::<Value>(
    &token,
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
    claims.retain(|k, _| k == "user" || k == "app" || k == "workflowRun");

    serde_json::from_value(r.claims)
      .context("Failed to parse identity from cookie")
  })
}
