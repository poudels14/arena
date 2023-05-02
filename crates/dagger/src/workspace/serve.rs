use anyhow::Result;
use arena_workspace::server::ServerOptions;
use arena_workspace::WorkspaceConfig;
use clap::Parser;
use std::path::Path;
use tracing::{info, Level};

#[derive(Parser, Debug)]
pub struct Command {
  /// Port to listen to
  #[arg(short, long, default_value_t = 8000)]
  pub port: u16,

  /// Directory of a workspace to serve; defaults to `${pwd}`
  #[arg(short, long)]
  pub dir: String,
}

impl Command {
  pub async fn execute(&self) -> Result<()> {
    let mut config = WorkspaceConfig::from_path(
      &Path::new(&self.dir).join("workspace.config.toml"),
    )?;
    // Note(sagar): just override the server entry
    config.server.entry = "./server/index.js".to_owned();

    let workspaces_dir = Path::new(&self.dir).join("build").to_path_buf();
    let workspace = arena_workspace::Workspace {
      registry: None,
      config,
      dir: workspaces_dir,
      heap_limits: None,
    };

    let handle = {
      let span = tracing::span!(Level::DEBUG, "starting workspace server");
      let _enter = span.enter();
      arena_workspace::server::serve(
        workspace,
        ServerOptions {
          port: 8000,
          ..Default::default()
        },
      )
      .await?
    };

    info!("Server started...");
    handle.wait_for_termination().await
  }
}
