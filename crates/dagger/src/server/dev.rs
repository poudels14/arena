use anyhow::Result;
use arena_server::{self, Server};
use chrono::Duration;
use clap::Parser;
use std::env;
use std::path::Path;

#[derive(Parser, Debug)]
pub struct DevCommand {
  /// Port to listen to
  #[arg(short, long, default_value_t = 9000)]
  pub port: u16,

  /// Directory of a workspace to serve; defaults to `${pwd}`
  #[arg(short, long)]
  pub dir: Option<String>,
}

impl DevCommand {
  pub async fn _execute(&self) -> Result<()> {
    let workspaces_dir = self
      .dir
      .as_ref()
      .map_or(env::current_dir().unwrap(), |p| Path::new(&p).to_path_buf());

    let server = Server::new(arena_server::Config {
      port: self.port,
      ttl: Duration::days(365),
      workspaces_dir,
    });

    server.start().await?;

    Ok(())
  }
}
