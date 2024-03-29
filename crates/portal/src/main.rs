mod config;
mod database;
mod server;
mod utils;
mod workspace;

use anyhow::bail;
use anyhow::Result;
use clap::Parser;
use colored::*;
use config::WorkspaceConfig;
use runtime::deno::core::v8;
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

  /// Reset all user data
  Reset,
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

  // Note: v8 platform has to be created before creating tokio runtime
  // spent days debugging why v8 runtime segfaulted :(
  let v8_platform = v8::new_default_platform(0, false).make_shared();
  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;

  let _ = rayon::ThreadPoolBuilder::new()
    .num_threads(3)
    .build()
    .unwrap();

  let term_now = Arc::new(AtomicBool::new(false));
  for sig in TERM_SIGNALS {
    flag::register_conditional_shutdown(*sig, 1, Arc::clone(&term_now))?;
    flag::register(*sig, Arc::clone(&term_now))?;
  }

  let mut signals = SignalsInfo::<SignalOnly>::new(TERM_SIGNALS)?;
  let signals_handle = signals.handle();

  let (shutdown_signal_tx, _) = broadcast::channel::<()>(10);
  let shutdown_signal_tx_clone = shutdown_signal_tx.clone();
  let handle = rt.spawn(async move {
    match args.command {
      Commands::Start(cmd) => {
        let res = cmd
          .execute(v8_platform.clone(), shutdown_signal_tx_clone.clone())
          .await;
        signals_handle.close();
        res?;
      }
      Commands::Reset => {
        let workspace_config =
          WorkspaceConfig::load().expect("Error loading config");
        let res = workspace_config.reset();
        signals_handle.close();
        res.expect("error resetting workspace");
      }
    };
    Ok::<(), anyhow::Error>(())
  });

  for _ in &mut signals {
    let _ = shutdown_signal_tx.send(());
    break;
  }

  let res = rt.block_on(handle).unwrap();
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
