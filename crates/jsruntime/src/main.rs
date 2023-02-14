mod buildtools;
mod core;
mod permissions;
mod utils;

use crate::core::{IsolatedRuntime, RuntimeConfig};
use crate::permissions::FileSystemPermissions;
use crate::permissions::PermissionsContainer;
use anyhow::Result;
use deno_core::resolve_url_or_path;
use std::collections::HashSet;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
  let mut runtime = IsolatedRuntime::new(RuntimeConfig {
    enable_console: true,
    module_loader_config: Some(crate::core::ModuleLoaderConfig {
      project_root: std::env::current_dir().unwrap(),
      ..Default::default()
    }),
    permissions: PermissionsContainer {
      fs: Some(FileSystemPermissions {
        allowed_read_paths: HashSet::from_iter(vec![
          Path::new("/").to_path_buf()
        ]),
        ..Default::default()
      }),
      ..Default::default()
    },
    ..Default::default()
  })?;

  let args: Vec<String> = std::env::args().collect();
  if args.len() > 1 {
    let main_module = resolve_url_or_path(&args[1]).unwrap();
    runtime.execute_main_module(&main_module).await.unwrap();
    runtime.run_event_loop().await.unwrap();
  }

  Ok(())
}
