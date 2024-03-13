use anyhow::Result;
use arenasql_cluster::schema::ADMIN_USERNAME;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{Request, StatusCode};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::MethodFilter;
use axum::{routing, Router};
use cloud::identity::Identity;
use common::axum::logger;
use dqs::arena::{App, ArenaRuntimeState, MainModule, Template};
use dqs::runtime::deno::RuntimeOptions;
use hyper::Body as HyperBody;
use runtime::deno::core::{v8, ModuleCode};
use runtime::env::{EnvVar, EnvironmentVariableStore};
use runtime::extensions::server::request::read_http_body_to_buffer;
use runtime::extensions::server::response::ParsedHttpResponse;
use runtime::extensions::server::{errors, HttpRequest, HttpServerConfig};
use runtime::permissions::PermissionsContainer;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot::Sender;
use tokio::sync::{mpsc, oneshot};
use tower::ServiceBuilder;
use url::Url;

use crate::utils::assets::PortalAppModules;
use crate::utils::templateloader::PortalTemplateLoader;
use crate::workspace::Workspace;

pub async fn start_workspace_server(
  v8_platform: v8::SharedRef<v8::Platform>,
  workspace: Workspace,
  request_stream_rx: Receiver<(HttpRequest, Sender<ParsedHttpResponse>)>,
) -> Result<()> {
  let module = MainModule::App {
    app: App {
      id: nanoid::nanoid!(),
      owner_id: None,
      template: Template {
        id: "workspace-desktop".to_owned(),
        version: env!("PORTAL_DESKTOP_WORKSPACE_VERSION").to_owned(),
      },
      workspace_id: "workspace-desktop".to_owned(),
    },
  };
  let mut runtime = dqs::runtime::deno::new(RuntimeOptions {
    id: nanoid::nanoid!(),
    db_pool: None,
    v8_platform,
    server_config: Some(HttpServerConfig::Stream(Arc::new(Mutex::new(Some(
      request_stream_rx,
    ))))),
    egress_address: None,
    egress_headers: None,
    heap_limits: None,
    permissions: PermissionsContainer::default(),
    exchange: None,
    acl_checker: None,
    state: ArenaRuntimeState {
      workspace_id: "workspace-1".to_owned(),
      module: module.clone(),
      env_variables: EnvironmentVariableStore::new(HashMap::from([
        env_var("DATABASE_URL", &workspace.database_url()),
        env_var("HOST", "http://localhost:4200"),
        env_var("DATABASE_HOST", "localhost"),
        env_var("DATABASE_PORT", &workspace.db_port.to_string()),
        env_var("DATABASE_NAME", "portal"),
        env_var("DATABASE_USER", ADMIN_USERNAME),
        env_var(
          "DATABASE_PASSWORD",
          &workspace.config.workspace_db_password.unwrap(),
        ),
        env_var("S3_ENDPOINT", ""),
        env_var("S3_ACCESS_KEY", ""),
        env_var("S3_ACCESS_SECRET", ""),
        env_var("REGISTRY_API_KEY", ""),
        env_var("REGISTRY_BUCKET", ""),
        env_var("JWT_SIGNING_SECRET", ""),
        env_var("LOGIN_EMAIL_SENDER", "invalid@desktop.sidecar.so"),
        env_var("RESEND_API_KEY", ""),
      ])),
    },
    identity: Identity::Unknown,
    module,
    template_loader: Arc::new(PortalTemplateLoader {}),
  })
  .await?;

  let mod_id = runtime
    .load_main_module(
      &Url::parse("builtin:///main").unwrap(),
      Some(ModuleCode::Arc(
        format!(
          r#"
        import {{ serve }} from "@arena/runtime/server";
        import server from "dqs:///@dqs/template/app";

        process.env.SSR = "true";
        serve(server);
        console.log("Server ready...");
        "#,
        )
        .into(),
      )),
    )
    .await?;

  let rx = runtime.mod_evaluate(mod_id);
  runtime.run_event_loop(Default::default()).await?;
  rx.await?;
  Ok(())
}

fn env_var(key: &str, value: &str) -> (String, EnvVar) {
  let id = nanoid::nanoid!();
  (
    id.clone(),
    EnvVar {
      id,
      key: key.to_owned(),
      value: Value::String(value.to_owned()),
      is_secret: false,
    },
  )
}

#[derive(Clone)]
pub struct WorkspaceRouter {
  workspace: Workspace,
  assets: Arc<PortalAppModules>,
  stream: mpsc::Sender<(HttpRequest, oneshot::Sender<ParsedHttpResponse>)>,
}

impl WorkspaceRouter {
  pub fn new(
    workspace: &Workspace,
    stream: mpsc::Sender<(HttpRequest, oneshot::Sender<ParsedHttpResponse>)>,
  ) -> Self {
    Self {
      workspace: workspace.to_owned(),
      assets: Arc::new(PortalAppModules::new()),
      stream,
    }
  }

  pub fn axum_router(self) -> Result<Router> {
    let app = Router::new()
      .route("/", routing::on(MethodFilter::all(), handle_app_index))
      .route(
        "/assets/apps/:templateId/:version/static/*path",
        routing::get(handle_static_asset_route),
      )
      .route(
        "/*path",
        routing::on(MethodFilter::all(), handle_app_routes),
      )
      .layer(
        ServiceBuilder::new().layer(middleware::from_fn(logger::middleware)),
      )
      .with_state(self.clone());
    Ok(app)
  }
}

pub async fn handle_static_asset_route(
  Path((template_id, version, path)): Path<(String, String, String)>,
  State(server): State<WorkspaceRouter>,
) -> impl IntoResponse {
  let asset = server
    .assets
    .get_asset(&format!("{}/{}/static/{}", template_id, version, path))
    .map_err(|_| errors::Error::ResponseBuilder)?;

  match asset {
    Some(asset) => {
      let mime = mime_guess::from_path(path);
      return Ok(
        Response::builder()
          .status(StatusCode::OK)
          .header("Content-Type", mime.first_raw().unwrap_or("text/plain"))
          .body(HyperBody::from(asset))?,
      );
    }
    None => Err(errors::Error::ResponseBuilder),
  }
}

pub async fn handle_app_routes(
  Path(path): Path<String>,
  Query(search_params): Query<Vec<(String, String)>>,
  State(server): State<WorkspaceRouter>,
  req: Request<Body>,
) -> impl IntoResponse {
  pipe_app_request(&server, path, search_params, req).await
}

pub async fn handle_app_index(
  Query(search_params): Query<Vec<(String, String)>>,
  State(server): State<WorkspaceRouter>,
  req: Request<Body>,
) -> impl IntoResponse {
  pipe_app_request(&server, "/".to_owned(), search_params, req).await
}

#[tracing::instrument(skip_all, err, level = "trace")]
pub async fn pipe_app_request(
  server: &WorkspaceRouter,
  path: String,
  search_params: Vec<(String, String)>,
  mut req: Request<Body>,
) -> Result<Response, errors::Error> {
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

  let mut headers = req
    .headers()
    .iter()
    .filter(|(name, _)| name.as_str() == "referer")
    .map(|(name, value)| (name.to_string(), value.to_str().unwrap().to_owned()))
    .collect::<Vec<(String, String)>>();

  headers.push((
    "x-portal-user".to_owned(),
    Identity::User {
      id: server.workspace.config.user_id.to_owned(),
      email: None,
    }
    .to_user_json()
    .expect("Error converting identity to JSON"),
  ));

  let body = read_http_body_to_buffer(&mut req).await?;
  let request = HttpRequest {
    method: req.method().to_string(),
    url: url.as_str().to_owned(),
    headers,
    body,
  };

  let (tx, rx) = oneshot::channel::<ParsedHttpResponse>();
  server
    .stream
    .send((request, tx))
    .await
    .map_err(|_| errors::Error::ServiceUnavailable)?;

  let res = rx.await.map_err(|_| errors::Error::ResponseBuilder)?;
  res.into_response().await
}
