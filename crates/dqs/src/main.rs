use std::env;
use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use anyhow::Result;
use colored::Colorize;
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
  /// The base dir where data like apps database should be temporarily mounted
  #[arg(long)]
  data_dir: String,
  /// The IP address that DQS should use for outgoing network requests
  /// from DQS JS runtime
  #[arg(long)]
  egress_addr: Option<String>,
}

fn main() -> Result<()> {
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

  let _ = required_env!("DQS_HOST");
  let registry_host = required_env!("REGISTRY_HOST");
  let registry_api_key = required_env!("REGISTRY_API_KEY");
  let _ = required_env!("DATABASE_URL");
  let _ = required_env!("JWT_SIGNINIG_SECRET");

  let data_dir = Path::new(&args.data_dir).to_path_buf().canonicalize()?;
  if !data_dir.is_dir() {
    panic!("data_dir should be a valid directory")
  }

  let dqs_cluster = DqsCluster::new(DqsClusterOptions {
    address: host,
    port,
    dqs_egress_addr,
    data_dir,
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
    tokio::sync::oneshot::channel::<()>();
  let handle: tokio::task::JoinHandle<Result<(), anyhow::Error>> =
    rt.spawn(async move {
      dqs_cluster.start_server(shutdown_signal_rx).await?;
      Ok(())
    });

  let term_now = Arc::new(AtomicBool::new(false));
  for sig in TERM_SIGNALS {
    flag::register_conditional_shutdown(*sig, 1, Arc::clone(&term_now))?;
    flag::register(*sig, Arc::clone(&term_now))?;
  }
  let mut signals = SignalsInfo::<SignalOnly>::new(TERM_SIGNALS)?;

  for _ in &mut signals {
    shutdown_signal_tx.send(()).unwrap();
    break;
  }

  rt.block_on(handle)?
}
