use anyhow::Result;
use arena_workspace::server::ServerOptions;
use clap::Parser;
use common::fs::has_file_in_file_tree;
use std::env;
use std::path::Path;
use tracing::{info, Level};

#[derive(Parser, Debug)]
pub struct Command {
  /// Port to listen to
  #[arg(short, long, default_value_t = 8000)]
  pub port: u16,

  /// Directory of a workspace to serve; defaults to `${pwd}`
  #[arg(short, long)]
  pub dir: Option<String>,
}

impl Command {
  pub async fn execute(&self) -> Result<()> {
    let cwd = env::current_dir()?;
    let workspaces_dir = self
      .dir
      .as_ref()
      .map_or_else(
        || has_file_in_file_tree(Some(&cwd), "workspace.config.toml"),
        |p| Some(Path::new(&p).to_path_buf()),
      )
      .unwrap_or(cwd);

    let workspace =
      arena_workspace::load::load(arena_workspace::load::Options {
        dir: env::current_dir()
          .unwrap()
          .join(workspaces_dir)
          .canonicalize()?,
        registry: None,
        ..Default::default()
      })
      .await?;

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

    info!("Dev server started...");
    handle.wait_for_termination().await
  }
}
