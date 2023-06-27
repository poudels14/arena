use anyhow::Result;
use common::dotenv;
use std::env;
use tracing_subscriber::prelude::*;
use tracing_tree::HierarchicalLayer;
mod cluster;
mod config;
mod db;
mod loaders;
mod server;
mod specifier;
use cluster::DqsCluster;

#[tokio::main]
async fn main() -> Result<()> {
  let subscriber = tracing_subscriber::registry()
    .with(tracing_subscriber::filter::EnvFilter::from_default_env())
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
  let dqs_cluster = DqsCluster::new()?;
  cluster::http::start_server(dqs_cluster, host, port).await?;

  Ok(())
}
