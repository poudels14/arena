use arenasql::chrono::{DateTime, Local};
use common::dirs;
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

#[napi]
pub fn get_todays_log_file_name() -> Result<String> {
  let local: DateTime<Local> = Local::now();
  dirs::portal()
    .map_err(|e| Error::new(Status::GenericFailure, format!("{:?}", e)))?
    .cache_dir()
    .join("logs")
    .join(&format!("portal.log.{}", local.format("%Y-%m-%d")))
    .as_os_str()
    .to_str()
    .map(|s| s.to_owned())
    .ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        format!("Error converting path to string"),
      )
    })
}
