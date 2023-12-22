use std::env;
use std::path::Path;

use anyhow::Result;
use clap::Parser;
use runtime::config::ArenaConfig;
use runtime::utils::fs::has_file_in_file_tree;
use tracing::info;

use crate::app::server::{self, ServerOptions};

#[derive(Parser, Debug)]
pub struct Command {
  /// Port to listen to
  #[arg(short, long, default_value_t = 8000)]
  pub port: u16,

  /// Directory of a workspace to serve; defaults to `${pwd}`
  #[arg(short, long)]
  pub dir: Option<String>,
}

impl Command {
  pub async fn execute(&self) -> Result<()> {
    let cwd = env::current_dir()?;
    let app_dir = self
      .dir
      .as_ref()
      .map_or_else(
        || has_file_in_file_tree(Some(&cwd), "package.json"),
        |p| Some(Path::new(&p).to_path_buf()),
      )
      .unwrap_or_else(|| cwd.clone());

    let config = ArenaConfig::load(&cwd.join(&app_dir).canonicalize()?)?;
    let server_entry = app_dir
      .join(config.server.entry)
      .canonicalize()
      .expect("Error canonicalizing server entry path");
    let server_entry = server_entry
      .to_str()
      .expect("Error getting server entry path as str");

    let server_options = ServerOptions {
      address: "0.0.0.0".to_owned(),
      port: self.port,
      transpile: true,
      root_dir: app_dir,
    };

    info!(
      "Startin dev server at {}:{}",
      server_options.address, server_options.port
    );
    server::start_server(
      server_options,
      &format!(
        r#"
          import {{ serve }} from "@arena/runtime/server";
          // Note(sagar): need to dynamically load the entry-server.tsx or
          // whatever the entry file is for the workspace so that it's
          // transpiled properly

          // Note(sagar): since this is running in server, set SSR = true
          process.env.SSR = "true";
          await import("file://{}").then(async ({{ default: m }}) => {{
            serve(m);
            console.log("[Workspace Server]: Listening to connections...");
          }});
          "#,
        server_entry
      ),
    )
    .await?;

    Ok(())
  }
}
