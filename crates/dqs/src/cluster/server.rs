use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::SystemTime;

use anyhow::{anyhow, bail, Result};
use cloud::pubsub::exchange::Exchange;
use cloud::rowacl::RowAclChecker;
use common::beam;
use deno_core::v8;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use runtime::env::EnvironmentVariableStore;
use runtime::extensions::server::response::ParsedHttpResponse;
use runtime::extensions::server::{HttpRequest, HttpServerConfig};
use runtime::permissions::PermissionsContainer;
use serde_json::Value;
use tokio::sync::{mpsc, oneshot, watch, Mutex};

use crate::arena::{ArenaRuntimeState, MainModule};
use crate::config::workspace::WorkspaceConfig;
use crate::db;
use crate::db::deployment::{app_deployments, Deployment};
use crate::db::workspace::workspaces;
use crate::loaders::registry::Registry;
use crate::loaders::RegistryTemplateLoader;
use crate::runtime::Command;
use crate::runtime::{deno::RuntimeOptions, ServerEvents};

static RUNTIME_COUNTER: Lazy<Arc<AtomicUsize>> =
  Lazy::new(|| Arc::new(AtomicUsize::new(1)));

#[derive(Debug, Clone)]
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
  /// Registry to be used to fetch bundled JS from
  pub registry: Registry,
  pub db_pool: Pool<ConnectionManager<PgConnection>>,
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
    let workspace_config = Self::load_workspace_config(&options)?;
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
    let thread_handle = thread.spawn(move || {
      let env_variables = match options.module.as_app() {
        Some(app) => ArenaRuntimeState::load_app_env_variables(
          &options.workspace_id,
          app,
          &mut options.db_pool.get()?,
        )
        .unwrap_or_default(),
        _ => EnvironmentVariableStore::new(HashMap::new()),
      };

      let state = ArenaRuntimeState {
        workspace_id: options.workspace_id.clone(),
        env_variables,
        module: options.module.clone(),
        registry: options.registry.clone(),
      };

      crate::runtime::server::start(
        RuntimeOptions {
          id: options.id,
          db_pool: options.db_pool.into(),
          v8_platform,
          server_config: HttpServerConfig::Stream(Arc::new(
            std::sync::Mutex::new(Some(http_requests_rx)),
          )),
          egress_address: options.dqs_egress_addr,
          heap_limits: workspace_config
            .runtime
            .heap_limit_mb
            .map(|limit| (10 * 1024 * 1024, limit * 1024 * 1204)),
          permissions,
          exchange,
          acl_checker,
          state,
          template_loader: Arc::new(RegistryTemplateLoader {
            registry: options.registry,
            module: options.module,
          }),
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

  #[tracing::instrument(skip_all, level = "trace")]
  pub async fn get_server_deployment(
    &self,
    id: &str,
  ) -> Result<Option<Deployment>> {
    let connection = &mut self.options.db_pool.get()?;
    let deployment = db::deployment::table
      .filter(app_deployments::id.eq(id.to_string()))
      .first::<Deployment>(connection)
      .optional()
      .map_err(|e| anyhow!("Failed to load DQS deployment from db: {}", e))?;

    Ok(deployment)
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
      started_at: SystemTime::now(),
      last_heartbeat_at: None,
      reboot_triggered_at: None,
    };

    let connection = &mut self.options.db_pool.get()?;

    // Since arenasql doesn't support ON CONFLICT, delete existing deployment
    // first
    diesel::delete(app_deployments::dsl::app_deployments)
      .filter(app_deployments::id.eq(deployment.id.to_string()))
      .execute(connection)?;
    diesel::insert_into(app_deployments::dsl::app_deployments)
      .values(&deployment)
      .execute(connection)
      .map_err(|e| anyhow!("Failed to update DQS deployment: {}", e))?;

    Ok(())
  }

  #[tracing::instrument(skip_all, err, level = "debug")]
  fn load_workspace_config(
    options: &DqsServerOptions,
  ) -> Result<WorkspaceConfig> {
    let app = options.module.as_app();
    if app.is_none() {
      return Ok(WorkspaceConfig::default());
    }

    let connection =
      &mut options.db_pool.get().map_err(|e| anyhow!("{}", e))?;

    let workspace = db::workspace::table
      .filter(workspaces::id.eq(options.workspace_id.to_string()))
      .filter(workspaces::archived_at.is_null())
      .first::<db::workspace::Workspace>(connection)
      .map_err(|e| anyhow!("Failed to load workspace from db: {}", e))?;

    serde_json::from_value::<WorkspaceConfig>(workspace.config)
      .map_err(|e| anyhow!("{}", e))
  }
}
