use anyhow::Result;
use colored::Colorize;
use common::dotenv;
use loaders::registry::Registry;
use std::env;
use std::path::Path;
use std::str::FromStr;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::prelude::*;
use tracing_tree::HierarchicalLayer;

mod arena;
mod cluster;
mod config;
mod db;
mod loaders;
mod server;
mod specifier;
use clap::Parser;
use cluster::{DqsCluster, DqsClusterOptions};

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
  /// The base dir where data like apps database should be temporarily mounted
  #[arg(long)]
  data_dir: String,
  /// The IP address that DQS should use for outgoing network requests
  /// from DQS JS runtime
  #[arg(long)]
  egress_addr: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
  let subscriber = tracing_subscriber::registry()
    .with(
      tracing_subscriber::filter::EnvFilter::from_default_env()
        // Note(sagar): filter out swc_* logs because they are noisy
        .add_directive(Directive::from_str("swc_=OFF").unwrap()),
    )
    .with(
      HierarchicalLayer::default()
        .with_indent_amount(2)
        .with_thread_names(true),
    );
  tracing::subscriber::set_global_default(subscriber).unwrap();

  dotenv::load_env(
    &env::var("MODE").unwrap_or(String::from("")),
    &env::current_dir().unwrap(),
  )
  .unwrap_or(vec![])
  .iter()
  .for_each(|(key, value)| env::set_var(key, value));

  let host = env::var("HOST").unwrap_or("0.0.0.0".to_owned());
  let port = env::var("PORT")
    .ok()
    .and_then(|p: String| p.parse().ok())
    .unwrap_or(8000);

  let args = Args::parse();
  let dqs_egress_addr = args
    .egress_addr
    .as_ref()
    .map(|addr| addr.parse())
    .transpose()?;

  let registry_host = env::var("REGISTRY_HOST").expect(&format!(
    "{}",
    "Missing environment variable `REGISTRY_HOST`".red()
  ));

  let registry_api_key = env::var("REGISTRY_API_KEY").expect(&format!(
    "{}",
    "Missing environment variable `REGISTRY_API_KEY`".red()
  ));

  let data_dir = Path::new(&args.data_dir).to_path_buf().canonicalize()?;
  if !data_dir.is_dir() {
    panic!("data_dir should be a valid directory")
  }

  let dqs_cluster = DqsCluster::new(DqsClusterOptions {
    dqs_egress_addr,
    data_dir,
    registry: Registry {
      host: registry_host,
      api_key: registry_api_key,
    },
  })?;

  cluster::http::start_server(dqs_cluster, host, port).await?;
  Ok(())
}
