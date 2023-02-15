use anyhow::Result;
use clap::Parser;
use deno_core::resolve_url_or_path;
use jsruntime::permissions::{FileSystemPermissions, PermissionsContainer};
use jsruntime::{IsolatedRuntime, RuntimeConfig};
use std::collections::HashSet;
use std::path::Path;

#[derive(Parser, Debug)]
pub struct Command {
  /// File to execute
  file: String,

  /// Whether to auto-transpile code; default true
  #[arg(short, long)]
  disable_transpile: bool,

  /// Whether to enable build tools in main runtime; default false
  #[arg(short, long)]
  enable_build_tools: bool,
}

impl Command {
  pub async fn execute(&self) -> Result<()> {
    let mut runtime = IsolatedRuntime::new(RuntimeConfig {
      enable_console: true,
      transpile: !self.disable_transpile,
      enable_build_tools: self.enable_build_tools,
      permissions: PermissionsContainer {
        fs: Some(FileSystemPermissions {
          allowed_read_paths: HashSet::from_iter(vec![
            // allow all files
            Path::new("/").to_path_buf(),
          ]),
          ..Default::default()
        }),
        ..Default::default()
      },
      ..Default::default()
    })?;

    let main_module = resolve_url_or_path(&self.file)?;
    runtime.execute_main_module(&main_module).await?;

    runtime.run_event_loop().await
  }
}
