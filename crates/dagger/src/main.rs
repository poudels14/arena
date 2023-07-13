mod exec;
mod format;
mod run;
mod server;
mod workspace;

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

  /// Execute inline JS; dagger exec "console.log('test')"
  Exec(exec::Command),

  /// Http server
  #[command(subcommand)]
  Server(server::Command),

  /// Arena workspace commands
  #[command(subcommand)]
  Workspace(workspace::Command),

  /// Format code using default formatters;
  /// e.g. `pnmp prettier -w` for js and `cargo fmt` rust
  #[command(alias = "fmt")]
  Format(format::Command),
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

  let args = Args::parse();
  debug!("Running cli with args: {:?}", args);

  dotenv::load_env(
    &env::var("MODE").unwrap_or(String::from("")),
    &env::current_dir().unwrap(),
  )
  .unwrap_or(vec![])
  .iter()
  .for_each(|(key, value)| env::set_var(key, value));

  let res: Result<()> = async {
    match args.command {
      Commands::Run(cmd) => cmd.execute().await?,
      Commands::Exec(cmd) => cmd.execute().await?,
      Commands::Server(cmd) => cmd.execute().await?,
      Commands::Workspace(cmd) => cmd.execute().await?,
      Commands::Format(cmd) => cmd.execute().await?,
    };
    Ok(())
  }
  .await;

  match res.as_ref() {
    Err(e) => {
      // colorize the error
      bail!(format!("{}", e).red().bold())
    }
    _ => Ok(()),
  }
}
