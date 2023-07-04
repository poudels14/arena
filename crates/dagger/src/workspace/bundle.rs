use anyhow::Result;
use clap::Parser;
use common::config::ArenaConfig;
use common::deno::extensions::{BuiltinExtensions, BuiltinModule};
use deno_core::resolve_url_or_path;
use jsruntime::permissions::{FileSystemPermissions, PermissionsContainer};
use jsruntime::{IsolatedRuntime, RuntimeOptions};
use std::collections::HashSet;
use url::Url;

#[derive(Parser, Debug)]
pub struct Command {
  /// Path to `bundle.config.js`
  #[arg(short('c'))]
  pub config: String,
}

impl Command {
  #[tracing::instrument(skip_all)]
  pub async fn execute(&self) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cwd = cwd.to_str().unwrap();
    let project_root = ArenaConfig::find_project_root()?;
    let mut runtime = IsolatedRuntime::new(RuntimeOptions {
      project_root: Some(project_root.clone()),
      config: Some(ArenaConfig::default()),
      enable_console: true,
      transpile: true,
      builtin_extensions: BuiltinExtensions::with_modules(vec![
        BuiltinModule::Fs,
        BuiltinModule::Node,
        BuiltinModule::Postgres,
        BuiltinModule::Transpiler,
        BuiltinModule::Resolver(project_root),
        BuiltinModule::Babel,
        BuiltinModule::Rollup,
        BuiltinModule::Bundler,
        BuiltinModule::CustomRuntimeModule(
          "dagger/bundler",
          include_str!("bundler.js"),
        ),
      ]),
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
      ..Default::default()
    })?;

    let config_file =
      resolve_url_or_path(&self.config, &std::env::current_dir()?)?;
    runtime
      .execute_main_module_code(
        &Url::parse("file://main").unwrap(),
        &format!(
          r#"
        import bundler from "{0}";
        import {{ loadConfig }} from "dagger/bundler";
        loadConfig().then(config => bundler(config));
        "#,
          config_file
        ),
      )
      .await?;
    runtime.run_event_loop().await
  }
}
