use anyhow::Result;
use common::arena;
use log::info;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Server {
  /// Server port
  pub port: i32,

  /// Workspace directory
  pub workspace_dir: PathBuf,
}

impl Server {
  pub async fn start(&self) -> Result<()> {
    info!("Starting server...");

    info!("Listening on port: {}", self.port);

    let config =
      arena::Config::from_path(&self.workspace_dir.join("arena.config.yaml"))?;

    println!("Config = {:?}", config);

    Ok(())
  }
}
