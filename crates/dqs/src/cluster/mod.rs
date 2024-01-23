use anyhow::{anyhow, bail, Result};
use cloud::pubsub::exchange::Exchange;
use colored::Colorize;
use dashmap::DashMap;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::{watch, Mutex};
use uuid::Uuid;

pub(crate) mod cache;
pub(crate) mod http;
pub(crate) mod server;
use self::cache::Cache;
use self::server::{DqsServer, DqsServerOptions, DqsServerStatus};
use crate::db;
use crate::db::deployment::app_deployments;
use crate::db::nodes::app_clusters;
use crate::loaders::registry::Registry;
use crate::runtime::server::ServerEvents;

#[derive(Clone)]
pub struct DqsClusterOptions {
  /// Cluster address
  pub address: String,
  /// Cluster port
  pub port: u16,
  /// The IP address that DQS should use for outgoing network requests
  /// from DQS JS runtime
  pub dqs_egress_addr: Option<IpAddr>,
  /// Registry to be used to fetch bundled JS from
  pub registry: Registry,
}

#[derive(Clone)]
pub struct DqsCluster {
  pub options: DqsClusterOptions,
  pub node_id: String,
  /// DqsServerStatus by server id
  pub servers: Arc<DashMap<String, DqsServerStatus>>,
  /// It seems like there's a race condition occationally that causes
  /// two instances of DQS server gets started for same app. So, use a
  /// global lock to make sure only one DQS server is spawned at a time
  pub spawn_lock: Arc<Mutex<usize>>,
  pub exchanges: Arc<DashMap<String, Exchange>>,
  pub db_pool: Pool<ConnectionManager<PgConnection>>,
  pub cache: Cache,
}

impl DqsCluster {
  pub fn new(options: DqsClusterOptions) -> Result<Self> {
    let db_pool = db::create_connection_pool()?;
    Ok(Self {
      options: options.clone(),
      node_id: Uuid::new_v4().to_string(),
      servers: Arc::new(DashMap::with_shard_amount(32)),
      spawn_lock: Arc::new(Mutex::new(0)),
      exchanges: Arc::new(DashMap::with_shard_amount(32)),
      db_pool: db_pool.clone(),
      cache: Cache::new(Some(db_pool)),
    })
  }

  #[tracing::instrument(skip_all, level = "debug")]
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

    let exchange = self.get_exchange(&options.workspace_id).await?;
    let (dqs_server, server_events) =
      DqsServer::spawn(options.clone(), Some(exchange)).await?;

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
    let server = self.servers.get(id).map(|s| s.clone());
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
            self.servers.remove(id);
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
      let _l = lock.lock().await;
    } else {
      let _l = self.spawn_lock.lock().await;

      // It's possible for two requests to get here at the same time
      // So, check if the server status has been added to the map before
      // doing do
      if !self.servers.contains_key(&id) {
        let lock = Arc::new(Mutex::new(false));
        let mut l = lock.lock().await;

        self
          .servers
          .insert(id.clone(), DqsServerStatus::Starting(lock.clone()));

        let cluster = self.clone();
        let result = async move {
          let (dqs_server, server_events) =
            cluster.spawn_dqs_server(options).await?;

          dqs_server.healthy().await?;
          dqs_server
            .update_server_deployment(&cluster.node_id)
            .await?;

          cluster.track_dqs_server(dqs_server, server_events).await;

          Ok::<(), anyhow::Error>(())
        }
        .await;

        *l = true;
        drop(l);

        match result {
          Ok(_) => {}
          Err(e) => {
            self.servers.remove(&id);
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

  pub async fn get_exchange(&self, workspace_id: &str) -> Result<Exchange> {
    let exchange = self.exchanges.get(workspace_id).map(|e| e.value().clone());

    match exchange {
      Some(e) => Ok(e),
      None => {
        let e = Exchange::new(workspace_id.to_string());
        self.exchanges.insert(workspace_id.to_string(), e.clone());

        let exchange = e.clone();
        // TODO(sagar): run all exchanges in a dedicated thread
        tokio::task::spawn(async move {
          let _ = exchange.run().await;
        });

        Ok(e)
      }
    }
  }

  #[tracing::instrument(skip_all, level = "debug")]
  pub fn mark_node_as_online(&self) -> Result<()> {
    let connection = &mut self.db_pool.get()?;
    let node = db::nodes::DqsNode {
      id: self.node_id.to_string(),
      host: self.options.address.clone(),
      port: self.options.port as i32,
      status: "ONLINE".to_owned(),
    };

    // Since arenasql doesn't support ON CONFLICT, delete existing app_clusters
    // first
    diesel::delete(app_clusters::dsl::app_clusters)
      .filter(app_clusters::id.eq(node.id.to_string()))
      .execute(connection)?;
    diesel::insert_into(app_clusters::dsl::app_clusters)
      .values(&node)
      .execute(connection)
      .map_err(|e| anyhow!("Failed to mark node as online: {}", e))?;
    Ok(())
  }

  #[tracing::instrument(skip_all, level = "debug")]
  pub fn mark_node_as_terminating(&self) -> Result<()> {
    let connection = &mut self.db_pool.get()?;
    diesel::update(app_clusters::dsl::app_clusters)
      .set(app_clusters::status.eq("TERMINATING".to_string()))
      .filter(app_clusters::id.eq(self.node_id.clone()))
      .execute(connection)
      .map_err(|e| anyhow!("Failed to mark node as offline: {}", e))?;

    diesel::delete(app_deployments::dsl::app_deployments)
      .filter(app_deployments::node_id.eq(self.node_id.clone()))
      .execute(connection)
      .map_err(|e| {
        anyhow!(
          "Failed to remove DQS deployments from terminating node: {}",
          e
        )
      })?;

    Ok(())
  }

  pub fn mark_node_as_terminated(&self) -> Result<()> {
    let connection = &mut self.db_pool.get()?;
    diesel::delete(app_clusters::dsl::app_clusters)
      .filter(app_clusters::id.eq(self.node_id.to_string()))
      .execute(connection)
      .map_err(|e| anyhow!("Failed to mark node as terminated: {}", e))?;
    Ok(())
  }

  async fn track_dqs_server(
    &self,
    dqs_server: DqsServer,
    mut server_events: watch::Receiver<ServerEvents>,
  ) {
    let dqs_server_id = dqs_server.options.id.clone();
    self.servers.insert(
      dqs_server.options.id.clone(),
      DqsServerStatus::Ready(dqs_server),
    );

    let cluster = self.clone();
    tokio::task::spawn(async move {
      while server_events.changed().await.is_ok() {
        let event = server_events.borrow().clone();
        match event {
          ServerEvents::Terminated(result) => {
            cluster.servers.remove(&dqs_server_id);
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
