mod dev;
use anyhow::Result;

#[derive(clap::Subcommand, Debug)]
pub enum Command {}

impl Command {
  pub async fn execute(&self) -> Result<()> {
    Ok(())
  }
}
