mod config;
mod core;
mod env;
mod extensions;
mod loaders;
mod permissions;
mod resolver;
mod utils;

use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread;

use anyhow::Result;
use deno_core::resolve_url_or_path;

use crate::config::RuntimeConfig;
use crate::core::{IsolatedRuntime, RuntimeOptions};
use crate::extensions::BuiltinExtensionProvider;
use crate::extensions::BuiltinModule;
use crate::loaders::{FileModuleLoader, ModuleLoaderOption};
use crate::permissions::FileSystemPermissions;
use crate::permissions::PermissionsContainer;
use crate::resolver::FilePathResolver;

#[tokio::main]
async fn main() -> Result<()> {
  let _ = thread::spawn(|| {
    let rt = tokio::runtime::Builder::new_current_thread()
      .enable_all()
      .build()
      .unwrap();

    let local = tokio::task::LocalSet::new();
    local.block_on(&rt, async {
      let mut config = RuntimeConfig::default();
      config.set_project_root(PathBuf::from("./"));
      let mut runtime = IsolatedRuntime::new(RuntimeOptions {
        enable_console: true,
        config,
        module_loader: Some(Rc::new(FileModuleLoader::new(
          ModuleLoaderOption {
            transpile: true,
            resolver: Rc::new(FilePathResolver::new(
              PathBuf::from("./"),
              Default::default(),
            )),
          },
        ))),
        enable_arena_global: true,
        permissions: PermissionsContainer {
          fs: Some(FileSystemPermissions {
            allowed_read_paths: HashSet::from_iter(vec!["/".to_owned()]),
            ..Default::default()
          }),
          ..Default::default()
        },
        builtin_extensions: vec![BuiltinModule::Node(None).get_extension()],
        ..Default::default()
      })?;

      runtime.run_event_loop().await.unwrap();
      let args: Vec<String> = std::env::args().collect();
      if args.len() > 1 {
        let main_module =
          resolve_url_or_path(&args[1], &std::env::current_dir()?).unwrap();
        println!("Executing main module: {}", main_module.to_string());
        runtime
          .execute_main_module(&main_module)
          .await
          .expect("Error executing main module");
        runtime.run_event_loop().await.unwrap();
      }
      Ok::<(), anyhow::Error>(())
    })
  })
  .join()
  .unwrap();
  Ok(())
}
