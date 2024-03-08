use anyhow::Result;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::Request;
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::MethodFilter;
use axum::{routing, Router};
use cloud::identity::Identity;
use cloud::CloudExtensionProvider;
use common::axum::logger;
use dqs::runtime::DQS_SNAPSHOT;
use runtime::config::RuntimeConfig;
use runtime::deno::core::Snapshot;
use runtime::extensions::server::request::read_http_body_to_buffer;
use runtime::extensions::server::response::ParsedHttpResponse;
use runtime::extensions::server::{errors, HttpRequest, HttpServerConfig};
use runtime::extensions::{
  BuiltinExtension, BuiltinExtensionProvider, BuiltinModule,
};
use runtime::permissions::{
  FileSystemPermissions, NetPermissions, PermissionsContainer, TimerPermissions,
};
use runtime::{IsolatedRuntime, RuntimeOptions};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot::Sender;
use tokio::sync::{mpsc, oneshot};
use tower::ServiceBuilder;
use tower::ServiceExt;
use tower_http::services::ServeFile;
use url::Url;

use crate::utils::moduleloader::{
  PortalModuleLoaderOptions, PortalModulerLoader,
};
use crate::workspace::Workspace;

pub async fn start_workspace_server(
  workspace: Workspace,
  request_stream_rx: Receiver<(HttpRequest, Sender<ParsedHttpResponse>)>,
) -> Result<()> {
  let mut builtin_extensions: Vec<BuiltinExtension> = vec![
    BuiltinModule::Env,
    BuiltinModule::Fs,
    BuiltinModule::Node(None),
    BuiltinModule::Postgres,
    BuiltinModule::Cloudflare,
  ]
  .iter()
  .map(|m| m.get_extension())
  .collect();

  builtin_extensions.push(
    BuiltinModule::HttpServer(HttpServerConfig::Stream(Arc::new(Mutex::new(
      Some(request_stream_rx),
    ))))
    .get_extension(),
  );

  builtin_extensions.push(
    BuiltinModule::UsingProvider(Rc::new(CloudExtensionProvider {
      publisher: None,
      acl_checker: None,
    }))
    .get_extension(),
  );

  let mut runtime = IsolatedRuntime::new(RuntimeOptions {
    startup_snapshot: Some(Snapshot::Static(DQS_SNAPSHOT)),
    config: RuntimeConfig {
      project_root: workspace.config.get_workspace_root_dir(),
      egress_headers: None,
      ..Default::default()
    },
    enable_console: true,
    enable_arena_global: true,
    builtin_extensions,
    module_loader: Some(Rc::new(PortalModulerLoader::new(
      PortalModuleLoaderOptions {},
    ))),
    permissions: PermissionsContainer {
      fs: Some(FileSystemPermissions::allow_all("/".into())),
      net: Some(NetPermissions::allow_all()),
      timer: Some(TimerPermissions::allow_hrtime()),
    },
    heap_limits: Some((
      // 10 MB
      10 * 1024 * 1024,
      // 2 GB
      2 * 1024 * 1024 * 1024,
    )),
    ..Default::default()
  })?;

  runtime
    .execute_main_module_code(
      &Url::parse("file:///arena/app-server")?,
      r#"
      import { serve } from "@arena/runtime/server";
      process.env.SSR = "true";
      serve({
        fetch() {
          console.log(process.env);
          return "NICE!"
        }
      })
      "#,
      true,
    )
    .await?;

  Ok(())
}

#[derive(Clone)]
pub struct WorkspaceRouter {
  app_template_dir: String,
  stream: mpsc::Sender<(HttpRequest, oneshot::Sender<ParsedHttpResponse>)>,
}

impl WorkspaceRouter {
  pub fn new(
    workspace: &Workspace,
    stream: mpsc::Sender<(HttpRequest, oneshot::Sender<ParsedHttpResponse>)>,
  ) -> Self {
    Self {
      app_template_dir: workspace
        .config
        .get_app_templates_dir()
        .to_str()
        .expect("getting app template dir")
        .to_owned(),
      stream,
    }
  }

  pub fn axum_router(self) -> Result<Router> {
    let app = Router::new()
      .route("/", routing::on(MethodFilter::all(), handle_app_index))
      .route("/assets/app/*path", routing::get(handle_asset_route))
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

pub async fn handle_asset_route(
  Path(path): Path<String>,
  State(server): State<WorkspaceRouter>,
  req: Request<Body>,
) -> impl IntoResponse {
  return ServeFile::new(format!("{}/{}", server.app_template_dir, path))
    .oneshot(req)
    .await
    .map_err(|_| errors::Error::ResponseBuilder);
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
    Identity::Unknown
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
