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
use cloud::identity::Identity;
use cloud::pubsub::EventSink;
use cloud::pubsub::Subscriber;
use colored::Colorize;
use common::axum::logger;
use http::header;
use http::HeaderValue;
use http::StatusCode;
use http::{Method, Request};
use hyper::Body;
use runtime::extensions::server::errors::Error;
use runtime::extensions::server::request::read_http_body_to_buffer;
use runtime::extensions::server::response::ParsedHttpResponse;
use runtime::extensions::server::{errors, HttpRequest};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::compression::predicate::NotForContentType;
use tower_http::compression::{CompressionLayer, DefaultPredicate, Predicate};
use tower_http::cors::{AllowOrigin, CorsLayer};
use url::Url;
use uuid::Uuid;

use super::auth::authenticate_user_using_headers;
use super::auth::parse_identity_from_header;
use super::{DqsCluster, DqsServerOptions};
use crate::arena::workflow::PluginWorkflow;
use crate::arena::workflow::WorkflowTemplate;
use crate::arena::MainModule;
use crate::cluster::server::DqsServerStatus;
use crate::db::workflow::WorkflowRun;
use crate::jsruntime::Command;

impl DqsCluster {
  #[allow(dead_code)]
  pub async fn start_server(
    &self,
    // additional router
    router: Option<Router>,
    mut shutdown_signal: broadcast::Receiver<()>,
  ) -> Result<()> {
    let compression_predicate = DefaultPredicate::new()
      .and(NotForContentType::new(mime::TEXT_EVENT_STREAM.as_ref()));

    let builder =
      ServiceBuilder::new().layer(middleware::from_fn(logger::middleware));
    // remove logger in desktop release
    #[cfg(all(feature = "desktop", not(debug_assertions)))]
    {
      let builder = ServiceBuilder::new();
    }

    let app = Router::new()
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
        builder
          .layer(CompressionLayer::new().compress_when(compression_predicate))
          .layer(
            CorsLayer::new()
              .allow_methods([Method::GET])
              .allow_origin(AllowOrigin::list(vec![])),
          ),
      )
      .with_state(self.clone());

    let app = match router {
      Some(router) => app.nest("/", router),
      None => app,
    };

    // TODO(sagar): listen on multiple ports so that a lot more traffic
    // can be served from single cluster
    let addr: SocketAddr = (
      Ipv4Addr::from_str(&self.options.address)?,
      self.options.port,
    )
      .into();

    tracing::info!(
      "{}",
      format!("Starting DQS cluster on port {}...", self.options.port)
        .yellow()
        .bold()
    );

    #[cfg(not(feature = "desktop"))]
    self.mark_node_as_online().await?;
    axum::Server::bind(&addr)
      .serve(app.into_make_service())
      .with_graceful_shutdown(async {
        shutdown_signal.recv().await.ok();

        #[cfg(not(feature = "desktop"))]
        let _ = self.mark_node_as_terminating();
      })
      .await?;

    #[cfg(not(feature = "desktop"))]
    self.mark_node_as_terminated().await?;

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
  Query(search_params): Query<Vec<(String, String)>>,
  State(cluster): State<DqsCluster>,
  req: Request<Body>,
) -> impl IntoResponse {
  pipe_app_request(cluster, app_id, "/".to_owned(), search_params, req).await
}

pub async fn handle_app_routes(
  Path((app_id, path)): Path<(String, String)>,
  Query(search_params): Query<Vec<(String, String)>>,
  State(cluster): State<DqsCluster>,
  req: Request<Body>,
) -> impl IntoResponse {
  let disable_cors = path == "_admin/healthy";
  pipe_app_request(cluster, app_id, path, search_params, req)
    .await
    .map(|mut res| {
      if disable_cors {
        res.headers_mut().insert(
          header::ACCESS_CONTROL_ALLOW_ORIGIN,
          HeaderValue::from_static("*"),
        );
      }
      res
    })
}

#[tracing::instrument(skip_all, err, level = "trace")]
pub async fn pipe_app_request(
  cluster: DqsCluster,
  app_id: String,
  path: String,
  search_params: Vec<(String, String)>,
  mut req: Request<Body>,
) -> Result<Response, errors::Error> {
  let (identity, app) = authenticate_user_using_headers(
    &cluster.cache,
    jwt_secret()?.as_str(),
    &app_id,
    &req,
  )
  .await?;

  let egress_headers = vec![
    (
      "x-portal-user".to_owned(),
      identity
        .to_user_json()
        .expect("Error converting identity to JSON"),
    ),
    (
      "x-portal-app".to_owned(),
      serde_json::to_string(&app).unwrap(),
    ),
  ];
  let dqs_server = cluster
    .get_or_spawn_dqs_server(DqsServerOptions {
      id: format!("app/{}", app_id),
      version: app.template.version.clone(),
      workspace_id: app.workspace_id.clone(),
      root: None,
      module: MainModule::App { app },
      db_pool: cluster.db_pool.clone(),
      dqs_egress_addr: cluster.options.dqs_egress_addr,
      template_loader: cluster.options.template_loader.clone(),
    })
    .await
    .map_err(|e| {
      tracing::warn!("{:?}", e);
      errors::Error::ServiceUnavailable
    })?;

  let url = {
    let mut url = Url::parse(&format!("http://0.0.0.0/")).unwrap();
    url.set_path(&path);
    {
      let mut params = url.query_pairs_mut();
      search_params.iter().for_each(|(key, value)| {
        params.append_pair(key, value);
      });
    }
    url
  };

  let body = read_http_body_to_buffer(&mut req).await?;
  let request = HttpRequest {
    method: req.method().to_string(),
    url: url.as_str().to_owned(),
    // don't pass in headers like `Cookie` that might contain
    // user auth credentials
    headers: egress_headers,
    body,
  };

  let (tx, rx) = oneshot::channel::<ParsedHttpResponse>();
  dqs_server
    .http_channel
    .send((request, tx))
    .await
    .map_err(|e| {
      tracing::warn!("{:?}", e);
      errors::Error::ServiceUnavailable
    })?;

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
  let (_, app) = authenticate_user_using_headers(
    &cluster.cache,
    jwt_secret()?.as_str(),
    &app_id,
    &req,
  )
  .await?;

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
      template_loader: cluster.options.template_loader.clone(),
    })
    .await
    .map_err(|e| {
      tracing::warn!("{:?}", e);
      errors::Error::ServiceUnavailable
    })?;

  dqs_server
    .http_channel
    .send((request, tx))
    .await
    .map_err(|e| {
      tracing::warn!("{:?}", e);
      errors::Error::ServiceUnavailable
    })?;

  let res = rx.await.map_err(|_| errors::Error::ResponseBuilder)?;
  res.into_response().await
}

#[allow(dead_code)]
pub async fn pipe_plugin_workflow_request(
  Path((workflow_id, path)): Path<(String, String)>,
  State(cluster): State<DqsCluster>,
  req: Request<Body>,
) -> Result<Response, errors::Error> {
  let identity = parse_identity_from_header(jwt_secret()?.as_str(), &req)
    .map_err(|_| errors::Error::NotFound)?;

  let wf_run: WorkflowRun =
    sqlx::query_as("SELECT * FROM workflow_runs WHERE id = $1")
      .bind(&workflow_id)
      .fetch_optional(&cluster.db_pool)
      .await
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
      ..
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
      template_loader: cluster.options.template_loader.clone(),
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
  let identity = parse_identity_from_header(jwt_secret()?.as_str(), &req)
    .unwrap_or(Identity::Unknown);

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

fn jwt_secret() -> Result<String> {
  Ok(std::env::var("JWT_SIGNING_SECRET")?)
}
