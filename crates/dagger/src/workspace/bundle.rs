use anyhow::Result;
use clap::Parser;
use deno_core::{ExtensionFileSource, ExtensionFileSourceCode};
use jsruntime::permissions::{FileSystemPermissions, PermissionsContainer};
use jsruntime::{IsolatedRuntime, RuntimeConfig};
use std::collections::HashSet;
use url::Url;

#[derive(Parser, Debug)]
pub struct Command {}

impl Command {
  #[tracing::instrument(skip_all)]
  pub async fn execute(&self) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cwd = cwd.to_str().unwrap();
    let mut runtime = IsolatedRuntime::new(RuntimeConfig {
      enable_console: true,
      enable_build_tools: true,
      enable_node_modules: true,
      permissions: PermissionsContainer {
        fs: Some(FileSystemPermissions {
          root: cwd.into(),
          allowed_read_paths: HashSet::from_iter(vec![
            // allow all files in current directory
            cwd.to_string(),
          ]),
          allowed_write_paths: HashSet::from_iter(vec![
            // allow all files in current directory
            cwd.to_string(),
          ]),
          ..Default::default()
        }),
        ..Default::default()
      },
      side_modules: vec![ExtensionFileSource {
        specifier: "@arena/workspace-server".to_owned(),
        code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
          "../../../../js/packages/workspace-server/dist/index.js"
        )),
      }],
      ..Default::default()
    })?;

    runtime
      .execute_main_module_code(
        &Url::parse("file://main").unwrap(),
        include_str!("./bundler.js"),
      )
      .await?;
    runtime.run_event_loop().await
  }
}
