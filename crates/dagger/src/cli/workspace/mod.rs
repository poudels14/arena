mod create;
mod serve;
use anyhow::Result;

#[derive(clap::Subcommand, Debug)]
pub enum Command {
  /// Serve a workspace
  Serve(serve::ServeCommand),

  /// Create a new workspace
  Create(create::CreateCommand),
}

impl Command {
  pub async fn execute(&self) -> Result<()> {
    match self {
      Self::Serve(cmd) => cmd.execute().await?,
      Self::Create(cmd) => cmd.execute().await?,
    }
    Ok(())
  }
}
