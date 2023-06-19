mod core;

use crate::core::{IsolatedRuntime, RuntimeOptions};
use anyhow::Result;
use common::deno::permissions::FileSystemPermissions;
use common::deno::permissions::PermissionsContainer;
use deno_core::resolve_url_or_path;
use std::collections::HashSet;

#[tokio::main]
async fn main() -> Result<()> {
  let mut runtime = IsolatedRuntime::new(RuntimeOptions {
    enable_console: true,
    permissions: PermissionsContainer {
      fs: Some(FileSystemPermissions {
        allowed_read_paths: HashSet::from_iter(vec!["/".to_owned()]),
        ..Default::default()
      }),
      ..Default::default()
    },
    ..Default::default()
  })?;

  let args: Vec<String> = std::env::args().collect();
  if args.len() > 1 {
    let main_module =
      resolve_url_or_path(&args[1], &std::env::current_dir()?).unwrap();
    runtime.execute_main_module(&main_module).await.unwrap();
    runtime.run_event_loop().await.unwrap();
  }

  Ok(())
}
