use crate::apps;
use crate::apps::App;
use crate::config::workspace::WorkspaceConfig;
use crate::db;
use crate::db::deployment::{dqs_deployments, Deployment};
use crate::db::workspace::workspaces;
use crate::loaders::registry::Registry;
use crate::server::entry::ServerEntry;
use crate::server::Command;
use crate::server::{RuntimeOptions, ServerEvents};
use anyhow::{anyhow, bail, Context, Result};
use common::beam;
use common::deno::extensions::server::response::ParsedHttpResponse;
use common::deno::extensions::server::{HttpRequest, HttpServerConfig};
use common::deno::extensions::BuiltinModule;
use deno_core::normalize_path;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use jsruntime::permissions::{FileSystemPermissions, PermissionsContainer};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashSet;
use std::net::IpAddr;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use tokio::sync::{mpsc, oneshot, watch, RwLock};

#[derive(Debug, Clone)]
pub struct DqsServerOptions {
  pub id: String,
  pub workspace_id: String,
  pub entry: ServerEntry,
  /// Pass the app if the Dqs server is for an app
  pub app: Option<App>,
  /// The IP address that DQS should use for outgoing network requests
  /// from DQS JS runtime
  pub dqs_egress_addr: Option<IpAddr>,
  /// Registry to be used to fetch bundled JS from
  pub registry: Registry,
  pub db_pool: Pool<ConnectionManager<PgConnection>>,
}

#[derive(Debug, Clone)]
pub enum DqsServerStatus {
  Starting(Arc<RwLock<bool>>),
  Ready(DqsServer),
}

#[derive(Debug, Clone)]
pub struct DqsServer {
  pub options: DqsServerOptions,
  pub http_channel:
    mpsc::Sender<(HttpRequest, oneshot::Sender<ParsedHttpResponse>)>,
  pub commands_channel: beam::Sender<Command, Value>,
}

impl DqsServer {
  #[tracing::instrument(skip_all, level = "trace")]
  pub async fn spawn(
    options: DqsServerOptions,
  ) -> Result<(DqsServer, watch::Receiver<ServerEvents>)> {
    let (http_requests_tx, http_requests_rx) = mpsc::channel(5);
    let (events_tx, mut receiver) = watch::channel(ServerEvents::Init);
    let permissions = Self::load_permissions(&options)?;
    let thread = thread::Builder::new().name(format!("dqs-[{}]", options.id));
    let options_clone = options.clone();
    let _thread_handle = thread.spawn(move || {
      let app_modules = match options.app.as_ref() {
        Some(app) => {
          let ext = RefCell::new(Some(apps::extension(app.clone())));
          vec![
            BuiltinModule::Custom(Rc::new(move || {
              ext.borrow_mut().take().unwrap()
            })),
            BuiltinModule::Custom(Rc::new(cloud::vectordb::extension)),
            BuiltinModule::Custom(Rc::new(cloud::llm::extension)),
            BuiltinModule::Custom(Rc::new(cloud::pdf::extension)),
            BuiltinModule::Custom(Rc::new(cloud::html::extension)),
          ]
        }
        None => vec![],
      };

      crate::server::start(
        RuntimeOptions {
          id: options.id,
          workspace_id: options.workspace_id,
          db_pool: options.db_pool.into(),
          server_config: HttpServerConfig::Stream(Rc::new(RefCell::new(
            http_requests_rx,
          ))),
          egress_address: options.dqs_egress_addr,
          modules: app_modules,
          permissions,
          app: options.app,
          registry: Some(options.registry),
          ..Default::default()
        },
        events_tx,
        options.entry.get_main_module()?,
      )
    });

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
  pub async fn update_server_deployment(
    &self,
    deployment: Deployment,
  ) -> Result<()> {
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
    if options.app.is_none() {
      return Ok(PermissionsContainer::default());
    }

    let app = options.app.as_ref().unwrap();
    let app_root_path = &app.root;
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
