use anyhow::Result;

mod s3loader;
mod serve;

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
