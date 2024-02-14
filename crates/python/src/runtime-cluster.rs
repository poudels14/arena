use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::str::FromStr;

use anyhow::{Context, Result};
use axum::routing;
use axum::Router;
use clap::Parser;
use tokio::signal;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::prelude::*;
use tracing_tree::HierarchicalLayer;

mod cluster;
mod fs;
mod grpc;
mod portal;
mod runtime;
mod server;
mod utils;

use cluster::Cluster;

/// Python runtime cluster
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
  /// Port
  #[arg(long)]
  port: u16,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
  let subscriber = tracing_subscriber::registry()
    .with(
      tracing_subscriber::filter::EnvFilter::from_default_env()
        .add_directive(Directive::from_str("tokio=OFF").unwrap())
        .add_directive(Directive::from_str("hyper=OFF").unwrap()),
    )
    .with(
      HierarchicalLayer::default()
        .with_indent_amount(2)
        .with_thread_names(true),
    );
  tracing::subscriber::set_global_default(subscriber).unwrap();

  let args = Args::parse();
  let cluster = Cluster::new();
  let app = Router::new()
    .route("/healthy", routing::get(healthy))
    .route("/create", routing::post(cluster::handlers::create_runtime))
    .route(
      "/exec/:runtime_id",
      routing::post(cluster::handlers::exec_code),
    )
    .with_state(cluster);

  let addr: SocketAddr = (
    Ipv4Addr::from_str("0.0.0.0").context("Unable to parse host address")?,
    args.port,
  )
    .into();
  println!("Python runtime cluster listening on 0.0.0.0:{}", args.port);
  axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .with_graceful_shutdown(async {
      let _ = signal::ctrl_c().await;
    })
    .await?;
  Ok(())
}

async fn healthy() -> &'static str {
  "ok"
}
