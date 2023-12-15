mod auth;
mod error;
pub(crate) mod pgwire;
mod server;

use ::pgwire::api::{MakeHandler, StatelessMakeHandler};
use ::pgwire::tokio::process_socket;
use clap::Parser;
use log::{info, LevelFilter};
use server::ClusterConfig;
use std::net::{Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::runtime::Builder;

use crate::server::ArenaSqlCluster;

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

/// arenasql-cluster
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
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

  /// Number of threads to use
  #[arg(long)]
  threads: Option<usize>,
}

fn main() {
  env_logger::Builder::new()
    .filter_level(LevelFilter::Info)
    .parse_default_env()
    .init();

  let args = Args::parse();
  let host = args.host.unwrap_or("0.0.0.0".to_owned());
  let port = args.port.unwrap_or(5432);
  let num_thread = args.threads.unwrap_or(num_cpus::get());

  let cluster = Arc::new(ArenaSqlCluster::new(
    &args.dir,
    ClusterConfig {
      cache_size_mb: args.cache_size,
    },
  ));
  let processor = Arc::new(StatelessMakeHandler::new(cluster.clone()));
  let authenticator = Arc::new(StatelessMakeHandler::new(cluster.clone()));

  let rt = Builder::new_multi_thread()
    .worker_threads(num_thread)
    .enable_all()
    .build()
    .unwrap();

  rt.block_on(async {
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
  })
}
