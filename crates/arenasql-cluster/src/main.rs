mod auth;
mod error;
mod extension;
mod init;
mod io;
mod pgwire;
mod schema;
mod server;
mod system;

use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use init::InitCluster;
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::iterator::exfiltrator::SignalOnly;
use signal_hook::iterator::SignalsInfo;
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;
use tracing_tree::HierarchicalLayer;

use server::ClusterOptions;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

/// Arena DB cluster
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
  /// Number of threads to use
  #[clap(short, long)]
  threads: Option<usize>,

  #[command(subcommand)]
  command: Commands,
}

#[derive(clap::Subcommand, Debug, Clone)]
enum Commands {
  /// Initialize Arena DB cluster
  Init(InitCluster),

  /// Start Arena DB cluster server
  Start(ClusterOptions),
}

fn main() {
  let subscriber = tracing_subscriber::registry()
    .with(
      tracing_subscriber::filter::EnvFilter::from_default_env()
        // Note(sagar): filter out noisy logs
        .add_directive(Directive::from_str("swc_=OFF").unwrap())
        .add_directive(Directive::from_str("tokio_=OFF").unwrap()),
    )
    .with(
      HierarchicalLayer::default()
        .with_indent_amount(2)
        .with_thread_names(true),
    )
    .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE));
  tracing::subscriber::set_global_default(subscriber).unwrap();

  let args = Args::parse();
  let num_thread = args.threads.unwrap_or(num_cpus::get());

  let rt = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(num_thread)
    .enable_all()
    .build()
    .unwrap();

  let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
  rt.spawn(async { listen_to_signal(shutdown_tx).await });

  let _ = rt.block_on(async {
    let handle: JoinHandle<Result<(), anyhow::Error>> = tokio::spawn(async {
      match args.command {
        Commands::Init(cmd) => cmd.execute().await,
        Commands::Start(cmd) => cmd.execute(shutdown_rx).await,
      }
    });

    if let Err(err) = handle.await.unwrap() {
      eprintln!("{:?}", err);
    }
  });

  rt.shutdown_background();
}

async fn listen_to_signal(shutdown_tx: oneshot::Sender<()>) -> Result<()> {
  let term_now = Arc::new(AtomicBool::new(false));
  for sig in TERM_SIGNALS {
    signal_hook::flag::register_conditional_shutdown(
      *sig,
      1,
      Arc::clone(&term_now),
    )
    .context("Error registering singal hook")?;
    signal_hook::flag::register(*sig, Arc::clone(&term_now))
      .context("Error registering singal hook")?;
  }
  let mut signals = SignalsInfo::<SignalOnly>::new(TERM_SIGNALS)
    .context("Error registering singal hook")?;

  for _ in &mut signals {
    let _ = shutdown_tx.send(());
    break;
  }
  Ok(())
}
