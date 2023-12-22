mod bundle;
mod dev;
pub(self) mod server;

use anyhow::Result;

#[derive(clap::Subcommand, Debug)]
pub enum Command {
  /// Serve an app in dev mode
  Dev(dev::Command),

  /// Bundle Arena workspace to client and server files
  Bundle(bundle::Command),
  // TODO
  // /// Serve a workspace in prod mode
  // Serve(serve::Command),
}

impl Command {
  pub async fn execute(&self) -> Result<()> {
    match self {
      Self::Dev(cmd) => cmd.execute().await?,
      Self::Bundle(cmd) => cmd.execute().await?,
    }
    Ok(())
  }
}
