use std::net::{Ipv4Addr, SocketAddr};
use std::ops::Add;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use anyhow::Context;
use anyhow::Result;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{Request, StatusCode};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::MethodFilter;
use axum::{routing, Router};
use cloud::dqs_runtime::DQS_SNAPSHOT;
use cloud::identity::Identity;
use cloud::CloudExtensionProvider;
use colored::Colorize;
use common::axum::logger;
use dqs::cluster::auth::{
  authenticate_user_using_headers, parse_identity_from_header,
};
use dqs::cluster::cache::Cache;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use runtime::config::RuntimeConfig;
use runtime::deno::core::{ModuleLoader, Snapshot};
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
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};
use tower::ServiceBuilder;
use url::Url;

#[derive(Clone)]
pub(super) struct ServerOptions {
  pub root_dir: PathBuf,
  // set this if ACL checker should be enabled
  pub app_id: Option<String>,
  pub allow_headers: Option<bool>,
  pub port: u16,
  pub address: String,
  pub heap_limit_mb: Option<usize>,
  pub builtin_modules: Vec<BuiltinModule>,
  pub module_loader: Option<Rc<dyn ModuleLoader>>,
}

pub(super) async fn start_js_server(
  options: ServerOptions,
  main_module: &str,
) -> Result<()> {
  let (stream_tx, stream_rx) = mpsc::channel(10);

  let db_pool = match options.app_id.is_some() {
    true => Some(dqs::db::create_connection_pool().await?),
    false => None,
  };

  let cache = Cache::new(db_pool);
  let server = AxumServer {
    app_id: options.app_id.clone(),
    cache: cache.clone(),
    allow_headers: options.allow_headers.clone().unwrap_or(false),
    stream: stream_tx,
  };

  let mut builtin_extensions: Vec<BuiltinExtension> = options
    .builtin_modules
    .iter()
    .map(|m| m.get_extension())
    .collect();

  builtin_extensions.push(
    BuiltinModule::HttpServer(HttpServerConfig::Stream(Arc::new(Mutex::new(
      Some(stream_rx),
    ))))
    .get_extension(),
  );

  let acl_checker = match options.app_id.as_ref() {
    Some(ref id) => Some(server.cache.get_app_acl_checker(&id).await?),
    _ => None,
  };
  builtin_extensions.push(
    BuiltinModule::UsingProvider(Rc::new(CloudExtensionProvider {
      publisher: None,
      acl_checker,
    }))
    .get_extension(),
  );

  let app = match options.app_id {
    Some(ref id) => cache.get_app(&id).await?,
    _ => None,
  };
  let app_identity = app
    .clone()
    .map(|app| Identity::App {
      id: app.id,
      owner_id: app.owner_id,
      system_originated: Some(true),
    })
    .unwrap_or_default();

  let mut identity_json = serde_json::to_value(&app_identity)?;
  if let Value::Object(_) = identity_json {
    // TODO: make exp <5mins and figure out a way to update default headers
    identity_json["exp"] = Value::Number(
      SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .add(Duration::from_secs(60 * 60 * 24 * 30))
        .as_secs()
        .into(),
    );
  }
  let auth_header = jsonwebtoken::encode(
    &Header::new(Algorithm::HS512),
    &identity_json,
    &EncodingKey::from_secret((&jwt_secret()).as_ref()),
  )
  .context("JWT encoding error")?;
  let mut runtime = IsolatedRuntime::new(RuntimeOptions {
    startup_snapshot: Some(Snapshot::Static(DQS_SNAPSHOT)),
    config: RuntimeConfig {
      project_root: options.root_dir.clone(),
      // TODO: make portal-authentication header domain specific
      // auth header should be sent only to portal apps
      egress_headers: Some(vec![(
        "x-portal-authentication".to_owned(),
        auth_header,
      )]),
      ..Default::default()
    },
    enable_console: true,
    enable_arena_global: true,
    builtin_extensions,
    module_loader: options.module_loader,
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

  let address = options.address.clone();
  let port = options.port;
  tokio::spawn(async move {
    server
      .start(address, port)
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
  allow_headers: bool,
  cache: Cache,
  stream: mpsc::Sender<(HttpRequest, oneshot::Sender<ParsedHttpResponse>)>,
}

impl AxumServer {
  async fn start(self, address: String, port: u16) -> Result<()> {
    let app = Router::new()
      .route(
        "/_admin/healthy",
        routing::get(|| async { (StatusCode::OK, "Ok") }),
      )
      .route("/", routing::on(MethodFilter::all(), handle_app_index))
      .route(
        "/*path",
        routing::on(MethodFilter::all(), handle_app_routes),
      )
      .layer(
        ServiceBuilder::new().layer(middleware::from_fn(logger::middleware)),
      )
      .with_state(self.clone());
    let addr: SocketAddr = (Ipv4Addr::from_str(&address)?, port).into();

    println!(
      "{}",
      format!("Starting app server port {}...", port)
        .yellow()
        .bold()
    );

    axum::Server::bind(&addr)
      .serve(app.into_make_service())
      .await?;
    Ok(())
  }
}

pub async fn handle_app_routes(
  Path(path): Path<String>,
  Query(search_params): Query<Vec<(String, String)>>,
  State(server): State<AxumServer>,
  req: Request<Body>,
) -> impl IntoResponse {
  pipe_app_request(&server, path, search_params, req).await
}

pub async fn handle_app_index(
  Query(search_params): Query<Vec<(String, String)>>,
  State(server): State<AxumServer>,
  req: Request<Body>,
) -> impl IntoResponse {
  pipe_app_request(&server, "/".to_owned(), search_params, req).await
}

#[tracing::instrument(skip_all, err, level = "trace")]
pub async fn pipe_app_request(
  server: &AxumServer,
  path: String,
  search_params: Vec<(String, String)>,
  mut req: Request<Body>,
) -> Result<Response, errors::Error> {
  let jwt_secret = jwt_secret();
  let (identity, app) = match &server.app_id {
    Some(app_id) => {
      let (identity, app) = authenticate_user_using_headers(
        &server.cache,
        jwt_secret.as_str(),
        app_id,
        &req,
      )
      .await?;
      (identity, Some(app))
    }
    _ => (
      parse_identity_from_header(jwt_secret.as_str(), &req)
        .unwrap_or(Identity::Unknown),
      None,
    ),
  };

  tracing::trace!("identity = {:?}", identity);
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
    .filter(|(name, _)| server.allow_headers || name.as_str() == "referer")
    .map(|(name, value)| (name.to_string(), value.to_str().unwrap().to_owned()))
    .collect::<Vec<(String, String)>>();

  headers.push((
    "x-portal-user".to_owned(),
    identity
      .to_user_json()
      .expect("Error converting identity to JSON"),
  ));
  if let Some(app) = app {
    headers.push((
      "x-portal-app".to_owned(),
      serde_json::to_string(&app).unwrap(),
    ));
  }

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

fn jwt_secret() -> String {
  std::env::var("JWT_SIGNING_SECRET")
    .expect("missing JWT_SIGNING_SECRET env variable")
}
