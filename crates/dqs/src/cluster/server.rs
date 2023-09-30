use anyhow::{anyhow, bail, Context, Result};
use cloud::pubsub::exchange::Exchange;
use common::beam;
use common::deno::extensions::server::response::ParsedHttpResponse;
use common::deno::extensions::server::{HttpRequest, HttpServerConfig};
use common::deno::resources::env_variable::EnvironmentVariableStore;
use deno_core::normalize_path;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use jsruntime::permissions::{FileSystemPermissions, PermissionsContainer};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::SystemTime;
use tokio::sync::{mpsc, oneshot, watch, Mutex};

use crate::arena::{ArenaRuntimeState, MainModule};
use crate::config::workspace::WorkspaceConfig;
use crate::db;
use crate::db::deployment::{dqs_deployments, Deployment};
use crate::db::workspace::workspaces;
use crate::loaders::registry::Registry;
use crate::runtime::Command;
use crate::runtime::{deno::RuntimeOptions, ServerEvents};

#[derive(Debug, Clone)]
pub struct DqsServerOptions {
  pub id: String,
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
    options: DqsServerOptions,
    exchange: Option<Exchange>,
  ) -> Result<(DqsServer, watch::Receiver<ServerEvents>)> {
    let (http_requests_tx, http_requests_rx) = mpsc::channel(200);
    let (events_tx, mut receiver) = watch::channel(ServerEvents::Init);
    let permissions = Self::load_permissions(&options)?;
    let thread = thread::Builder::new().name(format!("dqs-[{}]", options.id));
    let options_clone = options.clone();
    let thread_handle = thread.spawn(move || {
      let env_variables = match options.module.as_app() {
        Some(app) => ArenaRuntimeState::load_app_env_variables(
          &options.workspace_id,
          app,
          &mut options.db_pool.get()?,
        )?,
        _ => EnvironmentVariableStore::new(HashMap::new()),
      };

      let state = ArenaRuntimeState {
        workspace_id: options.workspace_id.clone(),
        root: options.root,
        env_variables,
        module: options.module.clone(),
        registry: options.registry.clone(),
      };

      crate::runtime::server::start(
        RuntimeOptions {
          id: options.id,
          db_pool: options.db_pool.into(),
          server_config: HttpServerConfig::Stream(Rc::new(RefCell::new(
            http_requests_rx,
          ))),
          egress_address: options.dqs_egress_addr,
          heap_limits: None,
          permissions,
          exchange,
          state,
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
        ServerEvents::Started(_isolate_handle, commands) => {
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
      .filter(dqs_deployments::id.eq(id.to_string()))
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
    diesel::insert_into(dqs_deployments::dsl::dqs_deployments)
      .values(&deployment)
      .on_conflict(dqs_deployments::id)
      .do_update()
      .set(&deployment)
      .execute(connection)
      .map_err(|e| anyhow!("Failed to update DQS deployment: {}", e))?;

    Ok(())
  }

  fn load_permissions(
    options: &DqsServerOptions,
  ) -> Result<PermissionsContainer> {
    let app = options.module.as_app();
    if app.is_none() {
      return Ok(PermissionsContainer::default());
    }

    let app_root_path = &options
      .root
      .clone()
      .context("App doesn't have access to file system")?;

    let connection =
      &mut options.db_pool.get().map_err(|e| anyhow!("{}", e))?;

    let workspace = db::workspace::table
      .filter(workspaces::id.eq(options.workspace_id.to_string()))
      .filter(workspaces::archived_at.is_null())
      .first::<db::workspace::Workspace>(connection)
      .map_err(|e| anyhow!("Failed to load workspace from db: {}", e))?;

    let workspace_config: WorkspaceConfig =
      serde_json::from_value(workspace.config).map_err(|e| anyhow!("{}", e))?;

    std::fs::create_dir_all(app_root_path)
      .context("Failed to create root directory for app")?;

    let allowed_read_paths = HashSet::from_iter(vec![app_root_path
      .to_str()
      .ok_or(anyhow!("Invalid app root path"))?
      .to_owned()]);

    let allowed_write_paths =
      vec![normalize_path(app_root_path.join("./db/")).to_str()]
        .iter()
        .filter(|p| p.is_some())
        .map(|p| p.map(|p| p.to_owned()))
        .collect::<Option<HashSet<String>>>()
        .unwrap_or_default();

    Ok(PermissionsContainer {
      fs: Some(FileSystemPermissions {
        root: app_root_path.clone(),
        allowed_read_paths,
        allowed_write_paths,
        ..Default::default()
      }),
      net: workspace_config.runtime.net_permissions,
      ..Default::default()
    })
  }
}
