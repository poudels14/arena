mod config;
mod database;
mod server;
mod workspace;

use anyhow::bail;
use anyhow::Result;
use clap::Parser;
use colored::*;
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::flag;
use signal_hook::iterator::exfiltrator::SignalOnly;
use signal_hook::iterator::SignalsInfo;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::prelude::*;
use tracing_tree::HierarchicalLayer;

/// Portal AI
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
  #[command(subcommand)]
  command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
  /// Start portal server
  Start(server::Command),
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
  let rt = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(num_cpus::get())
    .enable_all()
    .build()?;

  let (shutdown_signal_tx, _) = broadcast::channel::<()>(10);

  let local = tokio::task::LocalSet::new();
  let res = local.block_on(&rt, async {
    async {
      match args.command {
        Commands::Start(cmd) => cmd.execute(shutdown_signal_tx.clone()).await?,
      };
      Ok::<(), anyhow::Error>(())
    }
    .await
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

  match res {
    Err(e) => {
      if !e.to_string().contains("execution terminated") {
        // colorize the error
        eprintln!("Error: {}", format!("{:?}", e.to_string()).red().bold());
        bail!(e)
      }
      Ok(())
    }
    _ => Ok(()),
  }
}
