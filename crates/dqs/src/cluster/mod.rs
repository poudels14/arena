pub(crate) mod cache;
pub(crate) mod discovery;
pub(crate) mod http;
use self::cache::Cache;
use crate::db;
use crate::server::{Command, RuntimeOptions, ServerEvents};
use anyhow::Result;
use anyhow::{anyhow, Context};
use common::beam;
use common::deno::extensions::server::response::HttpResponse;
use common::deno::extensions::server::{HttpRequest, HttpServerConfig};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
pub struct DqsCluster {
  pub id: String,
  pub servers: Arc<Mutex<HashMap<String, DqsServer>>>,
  pub db_pool: Pool<ConnectionManager<PgConnection>>,
  pub cache: Cache,
}

#[derive(Debug, Clone)]
pub struct DqsServer {
  pub workspace_id: String,
  pub http_channel: mpsc::Sender<(HttpRequest, mpsc::Sender<HttpResponse>)>,
  pub commands_channel: beam::Sender<Command, Value>,
}

impl DqsCluster {
  pub fn new() -> Result<Self> {
    let db_pool = db::create_connection_pool()?;
    Ok(Self {
      id: Uuid::new_v4().to_string(),
      servers: Arc::new(Mutex::new(HashMap::new())),
      db_pool: db_pool.clone(),
      cache: Cache::new(Some(db_pool)),
    })
  }

  pub async fn spawn_dqs_server(&self, workspace_id: String) -> Result<()> {
    let (events_rx_tx, events_rx_rx) = oneshot::channel();
    let (stream_tx, stream_rx) = mpsc::channel(5);
    let db_pool = self.db_pool.clone();
    let workspace_id_clone = workspace_id.clone();
    let _thread_handle = thread::spawn(move || {
      crate::server::start(
        RuntimeOptions {
          workspace_id: workspace_id_clone,
          db_pool: db_pool.into(),
          server_config: HttpServerConfig::Stream(Rc::new(RefCell::new(
            stream_rx,
          ))),
          ..Default::default()
        },
        events_rx_tx,
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
              workspace_id.clone(),
              DqsServer {
                workspace_id: workspace_id.clone(),
                http_channel: stream_tx.clone(),
                commands_channel: commands,
              },
            );
            info!("[workspace = {}] DQS server started!", workspace_id);
            started_tx.take().map(|tx| tx.send(()));
          }
          Some(ServerEvents::Terminated(_result)) => {
            let mut servers = servers.lock().await;
            servers.remove(&workspace_id);
            info!("[workspace = {}] DQS server terminated!", workspace_id);
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

  pub async fn get_workspace_server(
    &self,
    workspace_id: &str,
  ) -> Option<DqsServer> {
    let servers = self.servers.lock().await;
    servers.get(workspace_id).map(|s| s.clone())
  }

  pub async fn get_or_spawn_workspace_server(
    &self,
    workspace_id: &str,
  ) -> Result<DqsServer> {
    match self.get_workspace_server(workspace_id).await {
      Some(s) => Ok(s),
      None => {
        self.spawn_dqs_server(workspace_id.to_string()).await?;
        self
          .get_workspace_server(workspace_id)
          .await
          .ok_or(anyhow!("Failed to start Workspace server"))
      }
    }
  }
}
