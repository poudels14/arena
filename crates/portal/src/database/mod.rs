use std::net::{Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use arenasql::pgwire::api::{MakeHandler, StatelessMakeHandler};
use arenasql::pgwire::tokio::process_socket;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, oneshot};

pub use arenasql_cluster::ArenaSqlCluster;

use arenasql_cluster::schema::ClusterManifest;
use arenasql_cluster::ArenaSqlClusterAuthenticator;

#[derive(clap::Parser, Debug, Clone)]
pub struct ArenasqlDatabase {}

impl ArenasqlDatabase {
  pub async fn start(
    self,
    manifest: ClusterManifest,
    mut shutdown_signal: broadcast::Receiver<()>,
    ready_signal: oneshot::Sender<u16>,
  ) -> Result<()> {
    let cluster = Arc::new(ArenaSqlCluster::load(manifest)?);
    let processor = Arc::new(StatelessMakeHandler::new(cluster.clone()));
    let authenticator = ArenaSqlClusterAuthenticator::new(cluster.clone());

    let addr: SocketAddr = (
      Ipv4Addr::from_str("127.0.0.1")
        .context("Unable to parse host address")?,
      0,
    )
      .into();
    let listener =
      TcpListener::bind(addr).await.context("TCP binding error")?;

    let shutdown_signal = shutdown_signal.recv();
    tokio::pin!(shutdown_signal);

    let port = listener.local_addr().expect("getting db port").port();
    ready_signal.send(port).expect("Failed to start database");
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
