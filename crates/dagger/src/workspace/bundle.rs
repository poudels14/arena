use anyhow::Result;
use clap::{value_parser, Parser};
use common::config::ArenaConfig;
use common::deno::extensions::{BuiltinExtensions, BuiltinModule};
use jsruntime::permissions::{FileSystemPermissions, PermissionsContainer};
use jsruntime::{IsolatedRuntime, RuntimeConfig};
use std::collections::HashSet;
use url::Url;

#[derive(Parser, Debug)]
pub struct Command {
  /// Whether to minify client bundle. Server bundle isn't minified
  #[arg(short('m'), long, value_parser=value_parser!(bool), default_value_t = true)]
  pub minify: bool,
}

impl Command {
  #[tracing::instrument(skip_all)]
  pub async fn execute(&self) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cwd = cwd.to_str().unwrap();
    let project_root = ArenaConfig::find_project_root()?;
    let mut runtime = IsolatedRuntime::new(RuntimeConfig {
      project_root: Some(project_root.clone()),
      config: Some(ArenaConfig::default()),
      enable_console: true,
      builtin_extensions: BuiltinExtensions::with_modules(vec![
        BuiltinModule::Fs,
        BuiltinModule::Node,
        BuiltinModule::Postgres,
        BuiltinModule::Transpiler,
        BuiltinModule::Resolver(project_root),
        BuiltinModule::Babel,
        BuiltinModule::Rollup,
        BuiltinModule::CustomRuntimeModule(
          "@arena/workspace-server/builder",
          include_str!(
            "../../../../js/packages/workspace-server/dist/builder.js"
          ),
        ),
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

    runtime
      .execute_main_module_code(
        &Url::parse("file://main").unwrap(),
        &format!(
          r#"
        import {{ bundle }} from "dagger/bundler";
        bundle({{
          minify: {0}
        }})
        "#,
          self.minify
        ),
      )
      .await?;
    runtime.run_event_loop().await
  }
}
