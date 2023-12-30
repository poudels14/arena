use std::rc::Rc;

use anyhow::Result;
use clap::Parser;
use cloud::CloudExtensionProvider;
use runtime::buildtools::{
  transpiler::BabelTranspiler, FileModuleLoader, FilePathResolver,
};
use runtime::config::{ArenaConfig, RuntimeConfig};
use runtime::deno::core::resolve_url_or_path;
use runtime::extensions::{BuiltinExtensionProvider, BuiltinModule};
use runtime::permissions::{
  FileSystemPermissions, NetPermissions, PermissionsContainer,
};
use runtime::{IsolatedRuntime, RuntimeOptions};

#[derive(Parser, Debug)]
pub struct Command {
  /// File to execute
  file: String,

  /// Whether to auto-transpile code; default is auto-transpile on
  #[arg(short, long)]
  disable_transpile: bool,

  /// Whether to enable build tools in main runtime; default false
  #[arg(short('b'), long)]
  enable_build_tools: bool,

  /// Enable @arena/cloud extension
  #[arg(long)]
  enable_cloud_ext: bool,

  /// The network address to use for outgoing network requests from JS runtime
  #[arg(long)]
  egress_addr: Option<String>,
}

impl Command {
  #[tracing::instrument(skip_all)]
  pub async fn execute(&self) -> Result<()> {
    let project_root = ArenaConfig::find_project_root()?;
    let arena_config = ArenaConfig::load(&project_root).unwrap_or_default();
    let mut builtin_modules = vec![
      BuiltinModule::Fs,
      BuiltinModule::Node(None),
      BuiltinModule::Env,
      BuiltinModule::Postgres,
      BuiltinModule::Sqlite,
    ];

    let resolver_config = arena_config
      .server
      .javascript
      .as_ref()
      .and_then(|js| js.resolve.clone())
      .unwrap_or_default();
    if self.enable_build_tools {
      builtin_modules.extend(vec![
        BuiltinModule::Resolver(project_root.clone(), resolver_config.clone()),
        BuiltinModule::Transpiler,
        BuiltinModule::Babel,
        BuiltinModule::Rollup,
      ])
    }

    if self.enable_cloud_ext {
      let cloud_ext = BuiltinModule::UsingProvider(Rc::new(
        CloudExtensionProvider::default(),
      ));
      builtin_modules.push(cloud_ext.clone());
    }

    let egress_addr = self
      .egress_addr
      .as_ref()
      .map(|addr| addr.parse())
      .transpose()?;
    let mut runtime = IsolatedRuntime::new(RuntimeOptions {
      config: RuntimeConfig {
        egress_addr,
        ..Default::default()
      },
      enable_arena_global: true,
      enable_console: true,
      module_loader: Some(Rc::new(FileModuleLoader::new(
        Rc::new(FilePathResolver::new(
          project_root.clone(),
          arena_config
            .server
            .javascript
            .and_then(|j| j.resolve)
            .unwrap_or_default(),
        )),
        Some(Rc::new(BabelTranspiler::new(
          project_root.clone(),
          resolver_config,
        ))),
      ))),
      builtin_extensions: builtin_modules
        .iter()
        .map(|m| m.get_extension())
        .collect(),
      permissions: PermissionsContainer {
        fs: Some(FileSystemPermissions::allow_all("/".into())),
        net: Some(NetPermissions::allow_all()),
        ..Default::default()
      },
      ..Default::default()
    })?;

    let main_module =
      resolve_url_or_path(&self.file, &std::env::current_dir()?)?;
    runtime.execute_main_module(&main_module).await?;
    runtime.run_event_loop().await
  }
}
