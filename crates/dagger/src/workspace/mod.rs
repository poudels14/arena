mod create;
mod dev;
use anyhow::Result;

#[derive(clap::Subcommand, Debug)]
pub enum Command {
  /// Create a new workspace
  Create(create::CreateCommand),

  /// Serve a workspace in dev mode
  Dev(dev::DevCommand),
}

impl Command {
  pub async fn execute(&self) -> Result<()> {
    match self {
      Self::Create(cmd) => cmd.execute().await?,
      Self::Dev(cmd) => cmd.execute().await?,
    }
    Ok(())
  }
}
