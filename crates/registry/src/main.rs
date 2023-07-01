use crate::registry::Registry;
use anyhow::Result;
use cache::FileStorage;
use common::dotenv;
use std::env;
use std::path::Path;
use tracing_subscriber::prelude::*;
use tracing_tree::HierarchicalLayer;
mod cache;
mod registry;
mod server;
mod template;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
  #[arg(long)]
  cache: String,
}

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
    .unwrap_or(9000);

  let args = Args::parse();
  print!("Using cache directory: {:?}", &args.cache);

  let registry = Registry::with_cache(FileStorage::new(Path::new(&args.cache)));
  server::start(host, port, registry).await
}
