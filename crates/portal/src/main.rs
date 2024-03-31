use anyhow::Result;
use clap::Parser;

use crate::portal::PortalArgs;

mod config;
mod database;
mod portal;
mod server;
mod utils;
mod workspace;

fn main() -> Result<()> {
  let args = PortalArgs::parse();
  portal::run_portal(args.command)
}
