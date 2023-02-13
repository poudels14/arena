use anyhow::Result;
use arena_workspace::server::ServerOptions;
use clap::Parser;
use std::env;
use std::path::Path;

#[derive(Parser, Debug)]
pub struct DevCommand {
  /// Port to listen to
  #[arg(short, long, default_value_t = 8000)]
  pub port: u16,

  /// Directory of a workspace to serve; defaults to `${pwd}`
  #[arg(short, long)]
  pub dir: Option<String>,
}

impl DevCommand {
  pub async fn execute(&self) -> Result<()> {
    let workspaces_dir = self
      .dir
      .as_ref()
      .map_or(env::current_dir().unwrap(), |p| Path::new(&p).to_path_buf());

    let workspace =
      arena_workspace::load::load(arena_workspace::load::Config {
        dir: workspaces_dir,
        registry: None,
        ..Default::default()
      })
      .await?;

    let handle = arena_workspace::server::serve(
      workspace,
      ServerOptions {
        port: 8000,
        ..Default::default()
      },
    )
    .await?;

    println!("Dev server started...");
    handle.wait_for_termination().await
  }
}
