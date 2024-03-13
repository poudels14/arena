use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;
use std::process;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use arenasql::pgwire::api::{MakeHandler, StatelessMakeHandler};
use arenasql::pgwire::tokio::process_socket;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

pub(crate) mod cluster;
mod execution;
pub(crate) mod storage;

use crate::pgwire::auth::ArenaSqlClusterAuthenticator;
use crate::schema::ClusterManifest;
pub use cluster::ArenaSqlCluster;

#[derive(clap::Parser, Debug, Clone)]
pub struct ClusterOptions {
  /// Database TCP host
  #[arg(long)]
  pub host: Option<String>,

  /// Database port
  #[arg(long)]
  pub port: Option<u16>,

  /// Path to config files
  /// Config file should be in .toml format
  #[arg(long)]
  pub config: String,
}

impl ClusterOptions {
  #[allow(dead_code)]
  pub async fn execute(
    self,
    mut shutdown_signal: oneshot::Receiver<()>,
  ) -> Result<()> {
    let manifest = std::fs::read_to_string(Path::new(&self.config))
      .context("Error reading cluster manifest")?;
    let manifest: ClusterManifest = toml::from_str(&manifest)?;

    let cluster = Arc::new(ArenaSqlCluster::load(manifest)?);
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

    tracing::info!(
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
