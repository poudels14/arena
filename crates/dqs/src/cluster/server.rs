use std::collections::HashMap;
use std::net::IpAddr;
use std::ops::Add;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};

use anyhow::{anyhow, bail, Context, Result};
use cloud::identity::Identity;
use cloud::pubsub::exchange::Exchange;
use cloud::rowacl::RowAclChecker;
use common::beam;
use deno_core::v8;
use derivative::Derivative;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use runtime::env::EnvironmentVariableStore;
use runtime::extensions::server::response::ParsedHttpResponse;
use runtime::extensions::server::{HttpRequest, HttpServerConfig};
use runtime::permissions::PermissionsContainer;
use serde_json::Value;
use sqlx::types::chrono::Utc;
use sqlx::{Pool, Postgres};
use tokio::sync::{mpsc, oneshot, watch, Mutex};

use crate::arena::{ArenaRuntimeState, MainModule};
use crate::config::workspace::WorkspaceConfig;
use crate::db::deployment::Deployment;
use crate::db::{self, deployment};
use crate::loaders::TemplateLoader;
use crate::runtime::Command;
use crate::runtime::{deno::RuntimeOptions, ServerEvents};

static RUNTIME_COUNTER: Lazy<Arc<AtomicUsize>> =
  Lazy::new(|| Arc::new(AtomicUsize::new(1)));

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct DqsServerOptions {
  pub id: String,
  /// App version or any other version
  pub version: String,
  pub workspace_id: String,
  pub root: Option<PathBuf>,
  pub module: MainModule,
  /// The IP address that DQS should use for outgoing network requests
  /// from DQS JS runtime
  pub dqs_egress_addr: Option<IpAddr>,

  #[derivative(Debug = "ignore")]
  pub template_loader: Arc<dyn TemplateLoader>,
  pub db_pool: Pool<Postgres>,
}

#[derive(Debug, Clone)]
pub enum DqsServerStatus {
  Starting(Arc<Mutex<bool>>),
  Ready(DqsServer),
}

#[derive(Debug, Clone)]
pub struct DqsServer {
  pub options: DqsServerOptions,
  pub http_channel:
    mpsc::Sender<(HttpRequest, oneshot::Sender<ParsedHttpResponse>)>,
  pub commands_channel: beam::Sender<Command, Value>,
  pub thread_handle: Arc<std::sync::Mutex<Option<JoinHandle<Result<()>>>>>,
}

impl DqsServer {
  #[tracing::instrument(skip_all, level = "trace")]
  pub async fn spawn(
    v8_platform: v8::SharedRef<v8::Platform>,
    options: DqsServerOptions,
    exchange: Option<Exchange>,
    acl_checker: Option<Arc<RwLock<RowAclChecker>>>,
  ) -> Result<(DqsServer, watch::Receiver<ServerEvents>)> {
    let (http_requests_tx, http_requests_rx) = mpsc::channel(200);
    let (events_tx, mut receiver) = watch::channel(ServerEvents::Init);
    let workspace_config = Self::load_workspace_config(&options).await?;
    let permissions = PermissionsContainer {
      net: workspace_config.runtime.net_permissions,
      ..Default::default()
    };

    let thread = thread::Builder::new().name(format!(
      "dqs-[{}]-{}",
      options.id,
      RUNTIME_COUNTER.fetch_add(1, Ordering::AcqRel)
    ));
    let options_clone = options.clone();

    let env_vars = match options.module.as_app() {
      Some(app) => ArenaRuntimeState::load_app_env_variables(
        &options.workspace_id,
        app,
        &options.db_pool,
      )
      .await
      .unwrap_or_default(),
      _ => HashMap::new(),
    };
    let thread_handle = thread.spawn(move || {
      let state = ArenaRuntimeState {
        workspace_id: options.workspace_id.clone(),
        env_variables: EnvironmentVariableStore::new(env_vars),
        module: options.module.clone(),
      };

      let identity = match &state.module {
        MainModule::App { app } => Identity::App {
          id: app.id.clone(),
          owner_id: app.owner_id.clone(),
          system_originated: None,
        },
        MainModule::PluginWorkflowRun { workflow } => Identity::WorkflowRun {
          id: workflow.id.to_string(),
          system_originated: None,
        },
        _ => Identity::Unknown,
      };

      let jwt_secret = std::env::var("JWT_SIGNING_SECRET")?;
      let mut identity_json = serde_json::to_value(&identity)?;
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
        &EncodingKey::from_secret((&jwt_secret).as_ref()),
      )
      .context("JWT encoding error")?;
      let egress_headers =
        Some(vec![("x-portal-authentication".to_owned(), auth_header)]);
      crate::runtime::server::start(
        RuntimeOptions {
          id: options.id,
          db_pool: options.db_pool.into(),
          v8_platform,
          server_config: Some(HttpServerConfig::Stream(Arc::new(
            std::sync::Mutex::new(Some(http_requests_rx)),
          ))),
          egress_address: options.dqs_egress_addr,
          egress_headers,
          heap_limits: workspace_config
            .runtime
            .heap_limit_mb
            .map(|limit| (10 * 1024 * 1024, limit * 1024 * 1204)),
          permissions,
          exchange,
          acl_checker,
          state,
          identity,
          module: options.module.clone(),
          template_loader: options.template_loader,
        },
        events_tx,
      )
    })?;

    loop {
      if receiver.changed().await.is_err() {
        bail!("Events stream closed");
      }
      let event = receiver.borrow().clone();
      match event {
        ServerEvents::Init => {}
        ServerEvents::Started(commands) => {
          return Ok((
            DqsServer {
              options: options_clone,
              http_channel: http_requests_tx,
              commands_channel: commands,
              thread_handle: Arc::new(std::sync::Mutex::new(Some(
                thread_handle,
              ))),
            },
            receiver,
          ));
        }
        ServerEvents::Terminated(result) => {
          bail!("{:?}", &result)
        }
      }
    }
  }

  #[tracing::instrument(skip_all, level = "trace")]
  pub async fn healthy(&self) -> Result<()> {
    let (tx, rx) = oneshot::channel::<ParsedHttpResponse>();
    let _ = self
      .http_channel
      .send((
        HttpRequest {
          method: "GET".to_owned(),
          url: format!("http://0.0.0.0/_admin/healthy"),
          headers: vec![],
          body: None,
        },
        tx,
      ))
      .await;
    let _ = rx.await;
    Ok(())
  }

  #[tracing::instrument(skip(self), level = "trace")]
  #[inline]
  pub async fn get_server_deployment(
    &self,
    id: &str,
  ) -> Result<Option<Deployment>> {
    deployment::get_deployment_with_id(&self.options.db_pool, id).await
  }

  #[tracing::instrument(skip_all, level = "trace")]
  pub async fn update_server_deployment(&self, node_id: &str) -> Result<()> {
    let app = self.options.module.as_app();
    let deployment = Deployment {
      id: self.options.id.clone(),
      node_id: node_id.to_string(),
      workspace_id: self.options.workspace_id.clone(),
      app_id: app.map(|a| a.id.clone()),
      app_template_id: app.map(|a| a.template.id.clone()),
      started_at: Utc::now().naive_utc(),
      last_heartbeat_at: None,
      reboot_triggered_at: None,
    };

    // Since arenasql doesn't support ON CONFLICT, delete existing deployment
    // first
    deployment::delete_deployment_with_id(
      &self.options.db_pool,
      &deployment.id,
    )
    .await?;
    deployment::insert_deployment(&self.options.db_pool, &deployment).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all, err, level = "debug")]
  async fn load_workspace_config(
    options: &DqsServerOptions,
  ) -> Result<WorkspaceConfig> {
    let app = options.module.as_app();
    if app.is_none() {
      return Ok(WorkspaceConfig::default());
    }

    let workspace: db::workspace::Workspace = sqlx::query_as(
      "SELECT * FROM workspaces WHERE id = $1 AND archived_at IS NULL",
    )
    .bind(&options.workspace_id)
    .fetch_one(&options.db_pool)
    .await?;

    serde_json::from_value::<WorkspaceConfig>(workspace.config)
      .map_err(|e| anyhow!("{}", e))
  }
}
