mod serve;
use anyhow::Result;

#[derive(clap::Subcommand, Debug)]
pub enum Command {
  /// Start a server
  Serve(serve::Command),
}

impl Command {
  pub async fn execute(&self) -> Result<()> {
    match self {
      Self::Serve(cmd) => cmd.execute().await?,
    }
    Ok(())
  }
}
