mod exec;
mod format;
mod run;
mod server;
mod workspace;

use anyhow::bail;
use anyhow::Result;
use chrono;
use clap::Parser;
use colored::*;
use log::debug;
use std::io::Write;

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

  /// Arena server commands
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
  env_logger::Builder::from_default_env()
    .filter_level(log::LevelFilter::Info)
    .parse_default_env()
    .format(|buf, record| {
      writeln!(
        buf,
        "{}\t{}",
        format!(
          "[{} {} {}:{}]",
          chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
          record.level().as_str().blue(),
          record.file().unwrap_or("<file/unknown>"),
          record.line().unwrap_or(0),
        )
        .yellow(),
        record.args()
      )
    })
    .init();

  let args = Args::parse();
  debug!("Running cli with args: {:?}", args);

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
