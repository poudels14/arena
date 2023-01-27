use crate::server::Server;
use anyhow::Result;
use clap::Parser;
use std::env;
use std::path::Path;

#[derive(Parser, Debug)]
pub struct ServeCommand {
  /// Port to listen to
  #[arg(short, long, default_value_t = 9000)]
  pub port: i32,

  /// Directory of a workspace to serve; defaults to `${pwd}`
  #[arg(short, long)]
  pub dir: Option<String>,
}

impl ServeCommand {
  pub async fn execute(&self) -> Result<()> {
    let workspace_dir = self
      .dir
      .as_ref()
      .map_or(env::current_dir().unwrap(), |p| Path::new(&p).to_path_buf());

    let server = Server {
      port: self.port,
      workspace_dir,
    };

    server.start().await?;

    Ok(())
  }
}
