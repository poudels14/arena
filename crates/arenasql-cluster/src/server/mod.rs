use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use log::info;
use pgwire::api::{MakeHandler, StatelessMakeHandler};
use pgwire::tokio::process_socket;
use tokio::net::TcpListener;

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
}

impl ServerOptions {
  pub async fn execute(self) -> Result<()> {
    let host = self.host.unwrap_or("0.0.0.0".to_owned());
    let port = self.port.unwrap_or(5432);

    let cluster = Arc::new(ArenaSqlCluster::load(ClusterOptions {
      dir: Arc::new(Path::new(&self.dir).to_path_buf()),
      cache_size_mb: self.cache_size,
    })?);

    let processor = Arc::new(StatelessMakeHandler::new(cluster.clone()));
    let authenticator = ArenaSqlClusterAuthenticator::new(cluster.clone());

    let addr: SocketAddr = (
      Ipv4Addr::from_str(&host).expect("Unable to parse host address"),
      port,
    )
      .into();
    let listener = TcpListener::bind(addr).await.expect("TCP binding error");

    info!("Listening to {}:{}", host, port);

    loop {
      let incoming_socket = listener.accept().await.unwrap();
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
