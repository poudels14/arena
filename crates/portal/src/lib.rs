use napi::{Error, Result, Status};
use napi_derive::napi;

mod config;
mod database;
mod portal;
mod server;
mod utils;
mod workspace;

#[napi]
pub fn start_portal_server(port: u16) -> Result<()> {
  portal::run_portal(portal::Command::Start(server::Command { port }))
    .map_err(|e| Error::new(Status::GenericFailure, format!("{:?}", e)))
}

#[napi]
pub fn reset_data() -> Result<()> {
  portal::run_portal(portal::Command::Reset)
    .map_err(|e| Error::new(Status::GenericFailure, format!("{:?}", e)))
}
