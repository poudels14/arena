mod dev;
mod new;
use anyhow::Result;

#[derive(clap::Subcommand, Debug)]
pub enum Command {
  /// Create a new workspace
  New(new::Command),

  /// Serve a workspace in dev mode
  Dev(dev::Command),
}

impl Command {
  pub async fn execute(&self) -> Result<()> {
    match self {
      Self::New(cmd) => cmd.execute().await?,
      Self::Dev(cmd) => cmd.execute().await?,
    }
    Ok(())
  }
}
