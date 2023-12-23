use std::rc::Rc;

use anyhow::Result;
use clap::Parser;
use runtime::buildtools::{FileModuleLoader, FilePathResolver};
use runtime::config::ArenaConfig;
use runtime::deno::core::resolve_url_or_path;
use runtime::extensions::{BuiltinExtensionProvider, BuiltinModule};
use runtime::permissions::{FileSystemPermissions, PermissionsContainer};
use runtime::{IsolatedRuntime, RuntimeOptions};
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

    let builtin_extensions = vec![
      BuiltinModule::Fs,
      BuiltinModule::Env,
      BuiltinModule::Node(None),
      BuiltinModule::Postgres,
      BuiltinModule::Transpiler,
      BuiltinModule::Resolver(project_root.clone()),
      BuiltinModule::Babel,
      BuiltinModule::Rollup,
      BuiltinModule::Bundler,
    ]
    .into_iter()
    .map(|m| m.get_extension())
    .collect();

    let mut runtime = IsolatedRuntime::new(RuntimeOptions {
      enable_console: true,
      module_loader: Some(Rc::new(FileModuleLoader::new(
        Rc::new(FilePathResolver::new(project_root, Default::default())),
        None,
      ))),
      builtin_extensions,
      permissions: PermissionsContainer {
        fs: Some(FileSystemPermissions::allow_all(cwd.into())),
        ..Default::default()
      },
      ..Default::default()
    })?;

    let config_file =
      resolve_url_or_path(&self.config, &std::env::current_dir()?)?;
    runtime
      .execute_main_module_code(
        &Url::parse("file:///main").unwrap(),
        &format!(
          r#"
        import bundler from "{0}";
        bundler(Arena.config);
        "#,
          config_file
        ),
      )
      .await?;
    runtime.run_event_loop().await
  }
}
