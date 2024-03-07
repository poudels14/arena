use std::env;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use anyhow::{Context, Result};
use common::{dotenv, required_env};
use loaders::registry::Registry;
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::flag;
use signal_hook::iterator::exfiltrator::SignalOnly;
use signal_hook::iterator::SignalsInfo;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::prelude::*;
use tracing_tree::HierarchicalLayer;

mod arena;
mod cluster;
mod config;
mod db;
mod loaders;
mod runtime;
mod specifier;
use clap::Parser;
use cluster::{DqsCluster, DqsClusterOptions};

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
  /// The IP address that DQS should use for outgoing network requests
  /// from DQS JS runtime
  #[arg(long)]
  egress_addr: Option<String>,

  /// Path to env file
  #[arg(long)]
  env_file: Option<String>,
}

fn main() -> Result<()> {
  let subscriber = tracing_subscriber::registry()
    .with(
      tracing_subscriber::filter::EnvFilter::from_default_env()
        // Note(sagar): filter out swc_* logs because they are noisy
        .add_directive(Directive::from_str("swc_=OFF").unwrap())
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
  let current_dir = env::current_dir().unwrap();

  let envs = if let Some(env_file) = args.env_file {
    let env_file_path = current_dir
      .join(&env_file)
      .canonicalize()
      .context(format!("error loading env file: {:?}", env_file))?;
    tracing::debug!("Loading env file: {:?}", env_file_path);
    dotenv::from_filename(&env_file_path)
  } else {
    dotenv::load_env(
      &env::var("MODE").unwrap_or(String::from("")),
      &current_dir,
    )
  };

  envs.unwrap_or_default().iter().for_each(|(key, value)| {
    tracing::debug!("Loading env: {}", key);
    env::set_var(key, value)
  });

  let host = env::var("HOST").unwrap_or("0.0.0.0".to_owned());
  let port = env::var("PORT")
    .ok()
    .and_then(|p: String| p.parse().ok())
    .unwrap_or(8000);

  let dqs_egress_addr = args
    .egress_addr
    .as_ref()
    .map(|addr| addr.parse())
    .transpose()?;

  let registry_host = required_env!("REGISTRY_HOST");
  let registry_api_key = required_env!("REGISTRY_API_KEY");
  let _ = required_env!("DATABASE_URL");
  let _ = required_env!("JWT_SIGNING_SECRET");

  let dqs_cluster = DqsCluster::new(DqsClusterOptions {
    address: host,
    port,
    dqs_egress_addr,
    registry: Registry {
      host: registry_host,
      api_key: registry_api_key,
    },
  })?;

  let rt = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(num_cpus::get())
    .enable_all()
    .build()?;

  let (shutdown_signal_tx, shutdown_signal_rx) =
    tokio::sync::broadcast::channel::<()>(10);
  let handle: tokio::task::JoinHandle<Result<(), anyhow::Error>> =
    rt.spawn(async move {
      dqs_cluster
        .start_server(None, shutdown_signal_rx)
        .await
        .unwrap();
      Ok(())
    });

  let term_now = Arc::new(AtomicBool::new(false));
  for sig in TERM_SIGNALS {
    flag::register_conditional_shutdown(*sig, 1, Arc::clone(&term_now))?;
    flag::register(*sig, Arc::clone(&term_now))?;
  }
  let mut signals = SignalsInfo::<SignalOnly>::new(TERM_SIGNALS)?;

  for _ in &mut signals {
    let _ = shutdown_signal_tx.send(());
    break;
  }

  rt.block_on(handle)?
}
