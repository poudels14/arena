use std::collections::BTreeMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::MethodFilter;
use axum::{routing, Router};
use cloud::CloudExtensionProvider;
use colored::Colorize;
use dqs::cluster::auth::authenticate_user_using_headers;
use dqs::cluster::cache::Cache;
use runtime::buildtools::{transpiler::BabelTranspiler, FileModuleLoader};
use runtime::config::{ArenaConfig, RuntimeConfig};
use runtime::extensions::server::request::read_http_body_to_buffer;
use runtime::extensions::server::response::ParsedHttpResponse;
use runtime::extensions::server::{errors, HttpRequest, HttpServerConfig};
use runtime::extensions::{
  BuiltinExtension, BuiltinExtensionProvider, BuiltinModule,
};
use runtime::permissions::{
  FileSystemPermissions, NetPermissions, PermissionsContainer, TimerPermissions,
};
use runtime::resolver::FilePathResolver;
use runtime::{IsolatedRuntime, RuntimeOptions};
use tokio::sync::{mpsc, oneshot};
use url::Url;

#[derive(Debug, Clone)]
pub(super) struct ServerOptions {
  pub root_dir: PathBuf,
  // set this if ACL checker should be enabled
  pub app_id: Option<String>,
  pub config: ArenaConfig,
  pub port: u16,
  pub address: String,
  pub transpile: bool,
  pub heap_limit_mb: Option<usize>,
}

pub(super) async fn start_js_server(
  options: ServerOptions,
  main_module: &str,
) -> Result<()> {
  let resolver_config = options
    .config
    .server
    .javascript
    .as_ref()
    .and_then(|js| js.resolve.clone())
    .unwrap_or_default();

  let (stream_tx, stream_rx) = mpsc::channel(10);
  let mut builtin_modules = vec![
    BuiltinModule::Fs,
    BuiltinModule::Env,
    BuiltinModule::Node(None),
    BuiltinModule::Postgres,
    BuiltinModule::Resolver(resolver_config.clone()),
    BuiltinModule::Transpiler,
    BuiltinModule::HttpServer(HttpServerConfig::Stream(Arc::new(Mutex::new(
      Some(stream_rx),
    )))),
  ];

  if options.transpile {
    builtin_modules.extend(vec![BuiltinModule::Babel])
  }

  let mut builtin_extensions: Vec<BuiltinExtension> =
    builtin_modules.iter().map(|m| m.get_extension()).collect();
  builtin_extensions.push(
    BuiltinModule::UsingProvider(Rc::new(CloudExtensionProvider::default()))
      .get_extension(),
  );

  let mut runtime = IsolatedRuntime::new(RuntimeOptions {
    config: RuntimeConfig {
      project_root: options.root_dir.clone(),
      ..Default::default()
    },
    enable_console: true,
    enable_arena_global: true,
    builtin_extensions,
    module_loader: Some(Rc::new(FileModuleLoader::new(
      Rc::new(FilePathResolver::new(
        options.root_dir.clone(),
        options
          .config
          .server
          .javascript
          .clone()
          .and_then(|j| j.resolve)
          .unwrap_or_default(),
      )),
      Some(Rc::new(BabelTranspiler::new(resolver_config).await)),
    ))),
    permissions: PermissionsContainer {
      fs: Some(FileSystemPermissions::allow_all("/".into())),
      net: Some(NetPermissions::allow_all()),
      timer: Some(TimerPermissions::allow_hrtime()),
    },
    heap_limits: options
      .heap_limit_mb
      .map(|limit| (1024 * 1024 * limit / 10, 1024 * 1024 * limit)),
    ..Default::default()
  })?;

  tokio::spawn(async move {
    start_axum_server(options, stream_tx)
      .await
      .expect("Error starting axum server");
  });

  runtime
    .execute_main_module_code(
      &Url::parse("file:///arena/app-server")?,
      main_module,
      true,
    )
    .await
}

#[derive(Clone)]
pub struct AxumServer {
  app_id: Option<String>,
  cache: Cache,
  stream: mpsc::Sender<(HttpRequest, oneshot::Sender<ParsedHttpResponse>)>,
}

async fn start_axum_server(
  options: ServerOptions,
  stream_tx: mpsc::Sender<(HttpRequest, oneshot::Sender<ParsedHttpResponse>)>,
) -> Result<()> {
  let db_pool = options
    .app_id
    .as_ref()
    .map(|_| dqs::db::create_connection_pool())
    .transpose()?;
  let server = AxumServer {
    app_id: options.app_id.clone(),
    cache: Cache::new(db_pool),
    stream: stream_tx,
  };
  let app = Router::new()
    .route(
      "/_admin/healthy",
      routing::get(|| async { (StatusCode::OK, "Ok") }),
    )
    .route(
      "/*path",
      routing::on(MethodFilter::all(), handle_app_routes),
    )
    .with_state(server.clone());
  let addr: SocketAddr =
    (Ipv4Addr::from_str(&options.address)?, options.port).into();

  println!(
    "{}",
    format!("Starting app server port {}...", options.port)
      .yellow()
      .bold()
  );

  axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await?;
  Ok(())
}

pub async fn handle_app_routes(
  Path(path): Path<String>,
  Query(search_params): Query<BTreeMap<String, String>>,
  State(server): State<AxumServer>,
  req: Request<Body>,
) -> impl IntoResponse {
  pipe_app_request(&server, path, search_params, req).await
}

#[tracing::instrument(skip_all, err, level = "trace")]
pub async fn pipe_app_request(
  server: &AxumServer,
  path: String,
  search_params: BTreeMap<String, String>,
  mut req: Request<Body>,
) -> Result<Response, errors::Error> {
  if let Some(app_id) = &server.app_id {
    let jwt_secret = std::env::var("JWT_SIGNING_SECRET")
      .expect("missing JWT_SIGNING_SECRET env variable");
    let _ = authenticate_user_using_headers(
      &server.cache,
      jwt_secret.as_str(),
      app_id,
      &req,
    )
    .await?;
  }

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
    headers: vec![],
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
