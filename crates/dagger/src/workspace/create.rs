use anyhow::Result;
use clap::Parser;
use colored::*;
use std::env;
use std::path::Path;

#[derive(Parser, Debug)]
pub struct CreateCommand {
  /// Workspace name
  pub name: String,

  /// Directory to setup workspace in; defaults to `${pwd}`
  #[arg(short, long)]
  pub dir: Option<String>,
}

impl CreateCommand {
  pub async fn execute(&self) -> Result<()> {
    let dir = self
      .dir
      .as_ref()
      .map_or(env::current_dir().unwrap(), |p| Path::new(&p).to_path_buf())
      .join(&self.name);

    arena_workspace::clone::with_default_template(
      &arena_workspace::clone::Config {
        name: self.name.to_string(),
        dir,
      },
    )
    .await?;

    println!("{}", "New workspace created successfully!".green().bold());

    Ok(())
  }
}
