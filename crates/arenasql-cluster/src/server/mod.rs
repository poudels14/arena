use std::net::{Ipv4Addr, SocketAddr};
use std::process;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use arenasql::pgwire::api::{MakeHandler, StatelessMakeHandler};
use arenasql::pgwire::tokio::process_socket;
use log::info;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

pub(crate) mod cluster;
mod execution;
pub(crate) mod storage;

use crate::pgwire::auth::ArenaSqlClusterAuthenticator;
pub use cluster::ArenaSqlCluster;

#[derive(clap::Parser, Debug, Clone)]
pub struct ClusterOptions {
  /// Database TCP host
  #[arg(long)]
  pub host: Option<String>,

  /// Database port
  #[arg(long)]
  pub port: Option<u16>,

  /// Directory to store database files
  #[arg(long)]
  pub root: String,

  /// A JWT signing secret that's used to authorize queries
  /// that access non-admin databases.
  /// If it's not set, env variable `ARENA_JWT_SECRET` will be checked
  /// and if that's also not set, unauthorized error will be returned
  /// for those queries.
  #[arg(long)]
  pub jwt_secret: Option<String>,

  /// Cache size per database in MB
  #[arg(long("cache_size"), default_value_t = 10)]
  pub cache_size_mb: usize,

  /// Directory to backup database to
  /// If set, all the database that were opened by the cluster will be
  /// backed up to that directory periodically
  #[arg(long)]
  pub backup_dir: Option<String>,

  /// Directory to put a checkpoint of the databases to
  /// When cluster is terminated, all the databases that were opened will
  /// be checkpointed to that directory
  #[arg(long)]
  checkpoint_dir: Option<String>,
}

impl ClusterOptions {
  pub async fn execute(
    self,
    mut shutdown_signal: oneshot::Receiver<()>,
  ) -> Result<()> {
    let cluster = Arc::new(ArenaSqlCluster::load(&self)?);
    let processor = Arc::new(StatelessMakeHandler::new(cluster.clone()));
    let authenticator = ArenaSqlClusterAuthenticator::new(cluster.clone());

    let host = self.host.unwrap_or("0.0.0.0".to_owned());
    let port = self.port.unwrap_or(5432);
    let addr: SocketAddr = (
      Ipv4Addr::from_str(&host).context("Unable to parse host address")?,
      port,
    )
      .into();
    let listener =
      TcpListener::bind(addr).await.context("TCP binding error")?;

    info!(
      "Listening to {}:{} [process id = {}]",
      host,
      port,
      process::id()
    );

    loop {
      tokio::select! {
        _ = &mut shutdown_signal => {
          break;
        },
        socket = listener.accept() => {
          let incoming_socket = socket?;
          let authenticator_ref = authenticator.make();
          let processor_ref = processor.make();
          tokio::spawn(async move {
            process_socket(
              incoming_socket.0,
              None,
              authenticator_ref,
              processor_ref.clone(),
              processor_ref,
            )
            .await
          });
        }
      }
    }
    cluster.graceful_shutdown().await
  }
}
