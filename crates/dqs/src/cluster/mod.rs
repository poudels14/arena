use anyhow::{bail, Result};
use colored::Colorize;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{watch, Mutex, RwLock};
use uuid::Uuid;

pub(crate) mod cache;
pub(crate) mod http;
pub(crate) mod server;
use self::cache::Cache;
use self::server::{DqsServer, DqsServerOptions, DqsServerStatus};
use crate::db;
use crate::loaders::registry::Registry;
use crate::runtime::server::ServerEvents;

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
  /// DqsServerStatus by server id
  pub servers: Arc<RwLock<HashMap<String, DqsServerStatus>>>,
  pub db_pool: Pool<ConnectionManager<PgConnection>>,
  pub cache: Cache,
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
      servers: Arc::new(RwLock::new(HashMap::new())),
      db_pool: db_pool.clone(),
      cache: Cache::new(Some(db_pool)),
    })
  }

  #[tracing::instrument(skip_all, level = "trace")]
  pub async fn spawn_dqs_server(
    &self,
    options: DqsServerOptions,
  ) -> Result<(DqsServer, watch::Receiver<ServerEvents>)> {
    let dqs_server_id = options.id.clone();
    let server_root = options
      .root
      .as_ref()
      .and_then(|root| root.to_str().map(|s| s.to_owned()))
      .unwrap_or("None".to_owned());

    let (dqs_server, server_events) = DqsServer::spawn(options.clone()).await?;

    println!(
      "{}",
      format!(
        "[{}] DQS server started! [root: {}]",
        dqs_server_id, server_root
      )
      .yellow()
    );

    Ok((dqs_server, server_events))
  }

  #[tracing::instrument(skip(self), level = "trace")]
  pub async fn get_server_by_id(
    &self,
    id: &str,
  ) -> Result<Option<DqsServerStatus>> {
    let servers = self.servers.read().await;
    let server = servers.get(id).map(|s| s.clone());
    drop(servers);
    match &server {
      Some(s) => {
        if let DqsServerStatus::Ready(s) = s {
          let deployment = s.get_server_deployment(id).await?;
          let reboot_triggered_after_deployment = deployment
            .map(|d| match d.reboot_triggered_at {
              Some(triggered_at) => {
                // Note(sagar): if duration_since returs err, it means
                // triggered_at is before d.started_at
                triggered_at.duration_since(d.started_at).is_ok()
              }
              None => false,
            })
            .unwrap_or(false);
          if reboot_triggered_after_deployment {
            let mut servers = self.servers.write().await;
            servers.remove(id);
            return Ok(None);
          }
        }
        Ok(server)
      }
      None => Ok(None),
    }
  }

  #[tracing::instrument(skip_all, level = "trace")]
  pub async fn get_or_spawn_dqs_server(
    &self,
    options: DqsServerOptions,
  ) -> Result<DqsServer> {
    let id = options.id.clone();

    // If the server is starting, wait for the server to start
    // When starting the server, first set the status to starting
    // so that if another request comes in immediately after the
    // first request, it doesn't start another DQS server but waits
    // for the first request to spin up the DQS server.
    // try 3 times :shrug:
    let status = self.get_server_by_id(&id).await?;
    if let Some(DqsServerStatus::Ready(s)) = status {
      return Ok(s);
    } else if let Some(DqsServerStatus::Starting(lock)) = status {
      let _ = lock.lock().await;
    } else {
      let mut servers = self.servers.write().await;
      // It's possible for two requests to get here at the same time
      // So, check if the server status has been added to the map before
      // doing do
      if !servers.contains_key(&id) {
        let lock = Arc::new(Mutex::new(false));
        servers.insert(id.clone(), DqsServerStatus::Starting(lock.clone()));
        drop(servers);

        let cluster = self.clone();
        let result = tokio::task::spawn(async move {
          let mut l = lock.lock().await;
          let (dqs_server, server_events) =
            cluster.spawn_dqs_server(options).await?;

          dqs_server.healthy().await?;
          dqs_server.update_server_deployment(&cluster.id).await?;

          cluster.track_dqs_server(dqs_server, server_events).await;

          *l = true;
          drop(l);

          Ok::<(), anyhow::Error>(())
        })
        .await;

        match result {
          Ok(_) => {}
          Err(e) => {
            let mut servers = self.servers.write().await;
            servers.remove(&id);
            return Err(e.into());
          }
        }
      }
    }

    let status = self.get_server_by_id(&id).await?;
    if let Some(DqsServerStatus::Ready(s)) = status {
      return Ok(s);
    }
    // Note(sagar): if the read lock is acquired and the server isn't ready,
    // it means there was error starting the server
    println!("Failed to start Workspace server");
    bail!("Failed to start Workspace server");
  }

  async fn track_dqs_server(
    &self,
    dqs_server: DqsServer,
    mut server_events: watch::Receiver<ServerEvents>,
  ) {
    let dqs_server_id = dqs_server.options.id.clone();
    let mut servers = self.servers.write().await;
    servers.insert(
      dqs_server.options.id.clone(),
      DqsServerStatus::Ready(dqs_server),
    );
    drop(servers);

    let cluster = self.clone();
    tokio::task::spawn(async move {
      while server_events.changed().await.is_ok() {
        let event = server_events.borrow().clone();
        match event {
          ServerEvents::Terminated(result) => {
            let mut servers = cluster.servers.write().await;
            servers.remove(&dqs_server_id);
            println!(
              "{}",
              format!(
                "[{}] DQS server terminated! Caused by = {:?}",
                &dqs_server_id, &result
              )
              .red()
            );
            break;
          }
          _ => {}
        }
      }
    });
  }
}
