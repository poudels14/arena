use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::process;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use log::info;
use pgwire::api::{MakeHandler, StatelessMakeHandler};
use pgwire::tokio::process_socket;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

pub(crate) mod cluster;
mod execution;
mod storage;

use crate::pgwire::auth::ArenaSqlClusterAuthenticator;

use self::cluster::ClusterOptions;
pub use cluster::ArenaSqlCluster;

#[derive(clap::Parser, Debug, Clone)]
pub struct ServerOptions {
  /// Database TCP host
  #[arg(long)]
  host: Option<String>,

  /// Database port
  #[arg(long)]
  port: Option<u16>,

  /// Directory to store database files
  #[arg(long)]
  dir: String,

  /// Cache size per database in MB
  #[arg(long, default_value_t = 10)]
  cache_size: usize,

  /// Directory to backup database to
  /// If set, all the database that were opened by the cluster will be
  /// backed up to that directory periodically
  #[arg(long)]
  backup_dir: Option<String>,

  /// Directory to put a checkpoint of the databases to
  /// When cluster is terminated, all the databases that were opened will
  /// be checkpointed to that directory
  #[arg(long)]
  checkpoint_dir: Option<String>,
}

impl ServerOptions {
  pub async fn execute(
    self,
    mut shutdown_signal: oneshot::Receiver<()>,
  ) -> Result<()> {
    let host = self.host.unwrap_or("0.0.0.0".to_owned());
    let port = self.port.unwrap_or(5432);

    let cluster = Arc::new(ArenaSqlCluster::load(ClusterOptions {
      dir: Arc::new(Path::new(&self.dir).to_path_buf()),
      cache_size_mb: self.cache_size,
      backup_dir: self
        .backup_dir
        .map(|p| create_path_if_not_exists(&p))
        .transpose()?,
      checkpoint_dir: self
        .checkpoint_dir
        .map(|p| create_path_if_not_exists(&p))
        .transpose()?,
    })?);

    let processor = Arc::new(StatelessMakeHandler::new(cluster.clone()));
    let authenticator = ArenaSqlClusterAuthenticator::new(cluster.clone());

    let addr: SocketAddr = (
      Ipv4Addr::from_str(&host).expect("Unable to parse host address"),
      port,
    )
      .into();
    let listener = TcpListener::bind(addr).await.expect("TCP binding error");

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

fn create_path_if_not_exists(path: &str) -> Result<PathBuf> {
  let p = PathBuf::from(path);
  if !p.exists() {
    std::fs::create_dir_all(&p)
      .context(format!("Failed to create dir: {:?}", p))?;
  }
  p.canonicalize()
    .context(format!("Failed to canonicalize path: {:?}", p))
}
