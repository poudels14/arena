use anyhow::{anyhow, bail, Result};
use cloud::pubsub::exchange::Exchange;
use colored::Colorize;
use dashmap::DashMap;
use deno_core::v8;
use sqlx::{Pool, Postgres};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::{watch, Mutex};
use uuid::Uuid;

pub(crate) mod server;

pub mod auth;
pub mod cache;
pub mod http;

use self::cache::Cache;
use self::server::{DqsServer, DqsServerOptions, DqsServerStatus};
use crate::db::{self, nodes};
use crate::jsruntime::{Command, ServerEvents};
use crate::loaders::TemplateLoader;

#[derive(Clone)]
pub struct DqsClusterOptions {
  pub v8_platform: v8::SharedRef<v8::Platform>,
  /// Cluster address
  pub address: String,
  /// Cluster port
  pub port: u16,
  /// The IP address that DQS should use for outgoing network requests
  /// from DQS JS runtime
  pub dqs_egress_addr: Option<IpAddr>,

  pub template_loader: Arc<dyn TemplateLoader>,
}

#[derive(Clone)]
pub struct DqsCluster {
  pub options: DqsClusterOptions,
  pub node_id: String,
  /// DqsServerStatus by server id
  pub servers: Arc<DashMap<String, DqsServerStatus>>,
  v8_platform: v8::SharedRef<v8::Platform>,
  /// It seems like there's a race condition occationally that causes
  /// two instances of DQS server gets started for same app. So, use a
  /// global lock to make sure only one DQS server is spawned at a time
  pub spawn_lock: Arc<Mutex<usize>>,
  pub exchanges: Arc<DashMap<String, Exchange>>,
  pub db_pool: Pool<Postgres>,
  pub cache: Cache,
}

impl DqsCluster {
  pub fn new(
    options: DqsClusterOptions,
    db_pool: Pool<Postgres>,
  ) -> Result<Self> {
    Ok(Self {
      options: options.clone(),
      node_id: Uuid::new_v4().to_string(),
      v8_platform: options.v8_platform,
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
    #[allow(unused_variables)]
    let server_root = options
      .root
      .as_ref()
      .and_then(|root| root.to_str().map(|s| s.to_owned()))
      .unwrap_or("None".to_owned());

    let exchange = self.get_exchange(&options.workspace_id).await?;
    let acl_checker = match options.module.as_app() {
      Some(app) => Some(
        self
          .cache
          .get_app_acl_checker(&app.id)
          .await
          .unwrap_or_default(),
      ),
      _ => None,
    };
    let (dqs_server, server_events) = DqsServer::spawn(
      self.v8_platform.clone(),
      options.clone(),
      Some(exchange),
      acl_checker,
    )
    .await?;

    #[cfg(any(not(feature = "desktop"), debug_assertions))]
    println!(
      "{}",
      format!(
        "[{}] DQS server started! [root: {}]",
        options.id, server_root
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
              Some(triggered_at) => triggered_at > d.started_at,
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
    if let Some(DqsServerStatus::Ready(server)) = status {
      if options.version == server.options.version {
        return Ok(server);
      }

      // if the version of the server doesn't match, terminate old version
      // and start with the new version
      let _ = server.commands_channel.send(Command::Terminate).await;
    } else if let Some(DqsServerStatus::Starting(lock)) = status {
      let _l = lock.lock().await;
      if let Some(DqsServerStatus::Ready(s)) =
        self.get_server_by_id(&id).await?
      {
        return Ok(s);
      }
    }

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
        Ok(_) => {
          if let Some(DqsServerStatus::Ready(s)) =
            self.get_server_by_id(&id).await?
          {
            return Ok(s);
          }
        }
        Err(e) => {
          self.servers.remove(&id);
          return Err(e.into());
        }
      }
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
  pub async fn mark_node_as_online(&self) -> Result<()> {
    let node = db::nodes::DqsNode {
      id: self.node_id.to_string(),
      host: self.options.address.clone(),
      port: self.options.port as i32,
      status: "ONLINE".to_owned(),
    };

    // Since arenasql doesn't support ON CONFLICT, delete existing app_clusters
    // first
    nodes::delete_dqs_node_with_id(&self.db_pool, &node.id).await?;
    nodes::insert_dqs_node(&self.db_pool, &node)
      .await
      .map_err(|e| anyhow!("Failed to mark node as online: {}", e))?;
    Ok(())
  }

  #[tracing::instrument(skip_all, level = "debug")]
  pub async fn mark_node_as_terminating(&self) -> Result<()> {
    nodes::update_dqs_node_status_with_id(
      &self.db_pool,
      &self.node_id,
      "TERMINATING",
    )
    .await
    .map_err(|e| anyhow!("Failed to mark node as offline: {}", e))?;

    nodes::delete_dqs_node_with_id(&self.db_pool, &self.node_id)
      .await
      .map_err(|e| {
        anyhow!(
          "Failed to remove DQS deployments from terminating node: {}",
          e
        )
      })?;

    Ok(())
  }

  pub async fn mark_node_as_terminated(&self) -> Result<()> {
    nodes::delete_dqs_node_with_id(&self.db_pool, &self.node_id)
      .await
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
