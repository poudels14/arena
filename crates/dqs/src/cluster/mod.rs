pub(crate) mod cache;
pub(crate) mod discovery;
pub(crate) mod http;
use self::cache::Cache;
use crate::apps::{self, App};
use crate::db;
use crate::loaders::registry::Registry;
use crate::server::entry::ServerEntry;
use crate::server::{Command, RuntimeOptions, ServerEvents};
use anyhow::{anyhow, Context};
use anyhow::{bail, Result};
use colored::Colorize;
use common::beam;
use common::deno::extensions::server::response::ParsedHttpResponse;
use common::deno::extensions::server::{HttpRequest, HttpServerConfig};
use common::deno::extensions::BuiltinModule;
use deno_core::normalize_path;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use jsruntime::permissions::PermissionsContainer;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use tokio::sync::{mpsc, oneshot, Mutex};
use uuid::Uuid;

#[derive(Clone)]
pub struct DqsClusterOptions {
  /// The IP address that DQS should use for outgoing network requests
  /// from DQS JS runtime
  pub dqs_egress_addr: Option<IpAddr>,
  /// The base dir where data like apps database should be temporarily mounted
  pub data_dir: PathBuf,
  /// Registry to be used to fetch bundled JS from
  pub registry: Registry,
}

#[derive(Clone)]
pub struct DqsCluster {
  pub options: DqsClusterOptions,
  pub id: String,
  pub data_dir: PathBuf,
  /// DqsServer by server id
  pub servers: Arc<Mutex<HashMap<String, DqsServer>>>,
  pub db_pool: Pool<ConnectionManager<PgConnection>>,
  pub cache: Cache,
}

#[derive(Debug, Clone, Default)]
pub struct DqsServerOptions {
  pub id: String,
  pub workspace_id: String,
  pub entry: ServerEntry,
  /// Pass the app if the Dqs server is for an app
  pub app: Option<App>,
  pub permissions: PermissionsContainer,
}

#[derive(Debug, Clone)]
pub struct DqsServer {
  pub id: String,
  pub workspace_id: String,
  pub http_channel:
    mpsc::Sender<(HttpRequest, oneshot::Sender<ParsedHttpResponse>)>,
  pub commands_channel: beam::Sender<Command, Value>,
}

impl DqsCluster {
  pub fn new(options: DqsClusterOptions) -> Result<Self> {
    if !options.data_dir.is_absolute() {
      bail!("options.data_dir should be an absolute path");
    }

    let db_pool = db::create_connection_pool()?;
    Ok(Self {
      options: options.clone(),
      id: Uuid::new_v4().to_string(),
      data_dir: options.data_dir,
      servers: Arc::new(Mutex::new(HashMap::new())),
      db_pool: db_pool.clone(),
      cache: Cache::new(Some(db_pool)),
    })
  }

  pub async fn spawn_dqs_server(
    &self,
    options: DqsServerOptions,
  ) -> Result<()> {
    let (events_rx_tx, events_rx_rx) = oneshot::channel();
    let (stream_tx, stream_rx) = mpsc::channel(5);
    let cluster = self.clone();
    let db_pool = self.db_pool.clone();
    let workspace_id = options.workspace_id.clone();

    let options_clone = options.clone();
    let _thread_handle = thread::spawn(move || {
      let app_modules = match options_clone.app.clone() {
        Some(app) => {
          let ext = RefCell::new(Some(apps::extension(app)));
          vec![
            BuiltinModule::Custom(Rc::new(move || {
              ext.borrow_mut().take().unwrap()
            })),
            BuiltinModule::Custom(Rc::new(cloud::llm::extension)),
            BuiltinModule::Custom(Rc::new(cloud::pdf::extension)),
            BuiltinModule::Custom(Rc::new(cloud::vectordb::extension)),
          ]
        }
        None => vec![],
      };

      crate::server::start(
        RuntimeOptions {
          id: options_clone.id,
          workspace_id: options_clone.workspace_id,
          db_pool: db_pool.into(),
          server_config: HttpServerConfig::Stream(Rc::new(RefCell::new(
            stream_rx,
          ))),
          egress_address: cluster.options.dqs_egress_addr.clone(),
          modules: app_modules,
          permissions: options_clone.permissions.clone(),
          app: options_clone.app,
          registry: Some(cluster.options.registry.clone()),
          ..Default::default()
        },
        events_rx_tx,
        options_clone.entry.get_main_module()?,
      )
    });

    let servers = self.servers.clone();
    let (started_tx, started_rx) = oneshot::channel();
    let mut started_tx = Some(started_tx);
    tokio::task::spawn(async move {
      let mut receiver = events_rx_rx
        .await
        .context("Error listening to DQS server events")
        .unwrap();

      loop {
        match receiver.recv().await {
          Some(ServerEvents::Started(_isolate_handle, commands)) => {
            let mut servers = servers.lock().await;
            servers.insert(
              options.id.clone(),
              DqsServer {
                id: options.id.clone(),
                workspace_id: workspace_id.clone(),
                http_channel: stream_tx.clone(),
                commands_channel: commands,
              },
            );
            println!(
              "{}",
              format!(
                "[{}] DQS server started! [root: {}]",
                options.id,
                options
                  .app
                  .clone()
                  .and_then(|a| normalize_path(a.root)
                    .to_str()
                    .map(|s| s.to_owned()))
                  .unwrap_or("None".to_owned())
              )
              .yellow()
            );
            started_tx.take().map(|tx| tx.send(()));
          }
          Some(ServerEvents::Terminated(result)) => {
            let mut servers = servers.lock().await;
            servers.remove(&options.id);
            println!(
              "[{}] DQS server terminated!{}",
              options.id,
              result
                .err()
                .map(|e| format!(" Caused by = {}", e))
                .unwrap_or_default()
            );
            break;
          }
          _ => {
            break;
          }
        }
      }
    });

    started_rx.await?;
    Ok(())
  }

  pub async fn get_server_by_id(&self, id: &str) -> Option<DqsServer> {
    let servers = self.servers.lock().await;
    servers.get(id).map(|s| s.clone())
  }

  pub async fn get_or_spawn_dqs_server(
    &self,
    options: DqsServerOptions,
  ) -> Result<DqsServer> {
    let id = options.id.clone();
    match self.get_server_by_id(&id).await {
      Some(s) => Ok(s),
      None => {
        self.spawn_dqs_server(options).await?;
        self
          .get_server_by_id(&id)
          .await
          .ok_or(anyhow!("Failed to start Workspace server"))
      }
    }
  }
}
