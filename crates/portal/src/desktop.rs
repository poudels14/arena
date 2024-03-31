#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::thread;
use std::time::Duration;

use anyhow::Result;
use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};

use crate::portal::{Command, PortalArgs};

mod config;
mod database;
mod portal;
mod server;
mod utils;
mod workspace;

fn main() -> Result<()> {
  let matches = PortalArgs::command().try_get_matches();
  let command = match matches {
    Ok(_) => PortalArgs::parse().command,
    Err(e) => match e.kind() {
      ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
        Command::Start(server::Command::default())
      }
      _ => PortalArgs::parse().command,
    },
  };

  match command {
    // run desktop UI only on Start command
    Command::Start(_) => {
      tauri::Builder::default()
        .setup(|_app| {
          let _ = thread::spawn(|| portal::run_portal(command));
          // This is required since the UI doesn't load at all if the server
          // isn't listening when the request is made from the frontend
          wait_until_server_ready();
          Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    }
    _ => {
      let _ = portal::run_portal(command);
    }
  }

  Ok(())
}

fn wait_until_server_ready() {
  loop {
    let client = reqwest::blocking::Client::builder()
      .timeout(Duration::from_secs(10))
      .build()
      .expect("Error checking application status");
    let res = client.get("http://localhost:42690/_healthy").send();
    if res.is_ok() {
      break;
    }
    std::thread::sleep(Duration::from_millis(100));
  }
}
