mod app;
mod format;
mod run;
mod server;

use anyhow::bail;
use anyhow::Result;
use clap::Parser;
use colored::*;
use common::dotenv;
use std::env;
use std::str::FromStr;
use tracing::debug;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::prelude::*;
use tracing_tree::HierarchicalLayer;

/// Dagger cli
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
  #[command(subcommand)]
  command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
  /// Execute a js file
  Run(run::Command),

  /// Http server
  #[command(subcommand)]
  Server(server::Command),

  /// Arena app commands
  #[command(subcommand)]
  App(app::Command),

  /// Format code using default formatters;
  /// e.g. `pnmp prettier -w` for js and `cargo fmt` rust
  #[command(alias = "fmt")]
  Format(format::Command),
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

  let args = Args::parse();
  debug!("Running cli with args: {:?}", args);

  dotenv::load_env(
    &env::var("MODE").unwrap_or(String::from("")),
    &env::current_dir().unwrap(),
  )
  .unwrap_or(vec![])
  .iter()
  .for_each(|(key, value)| {
    tracing::debug!("Loading env: {}", key);
    env::set_var(key, value)
  });

  let rt = tokio::runtime::Builder::new_current_thread()
    .enable_io()
    .enable_time()
    .worker_threads(1)
    .build()?;

  let local = tokio::task::LocalSet::new();
  let res = local.block_on(&rt, async {
    async {
      match args.command {
        Commands::Run(cmd) => cmd.execute().await?,
        Commands::App(cmd) => cmd.execute().await?,
        Commands::Server(cmd) => cmd.execute().await?,
        Commands::Format(cmd) => cmd.execute().await?,
      };
      Ok::<(), anyhow::Error>(())
    }
    .await
  });

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
