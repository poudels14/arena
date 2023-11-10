use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Command {
  /// Code to execute
  code: String,
}

impl Command {
  pub async fn execute(&self) -> Result<()> {
    unimplemented!()
  }
}
