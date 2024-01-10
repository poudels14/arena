use std::rc::Rc;

use anyhow::{bail, Result};
use clap::Parser;
use cloud::CloudExtensionProvider;
use runtime::buildtools::transpiler::ModuleTranspiler;
use runtime::buildtools::{
  transpiler::BabelTranspiler, FileModuleLoader, FilePathResolver,
};
use runtime::config::{ArenaConfig, RuntimeConfig};
use runtime::extensions::{BuiltinExtensionProvider, BuiltinModule};
use runtime::permissions::{
  FileSystemPermissions, NetPermissions, PermissionsContainer, TimerPermissions,
};
use runtime::{IsolatedRuntime, RuntimeOptions};
use url::Url;

#[derive(Parser, Debug)]
pub struct Command {
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

  /// File or npm module to execute
  module: String,

  args: Vec<String>,
}

impl Command {
  #[tracing::instrument(skip_all)]
  pub async fn execute(&self) -> Result<()> {
    let project_root = ArenaConfig::find_project_root()?;
    let arena_config = ArenaConfig::load(&project_root).unwrap_or_default();

    let resolver_config = arena_config
      .server
      .javascript
      .as_ref()
      .and_then(|js| js.resolve.clone())
      .unwrap_or_default();
    let mut builtin_modules = vec![
      BuiltinModule::Fs,
      BuiltinModule::Node(None),
      BuiltinModule::Env,
      BuiltinModule::Postgres,
      BuiltinModule::Sqlite,
      // enable resolver/transpiler by default since this is dev env
      BuiltinModule::Resolver(resolver_config.clone()),
      BuiltinModule::Transpiler,
    ];

    if self.enable_build_tools {
      builtin_modules.extend(vec![BuiltinModule::Babel, BuiltinModule::Rollup])
    }

    if self.enable_cloud_ext {
      let cloud_ext = BuiltinModule::UsingProvider(Rc::new(
        CloudExtensionProvider::default(),
      ));
      builtin_modules.push(cloud_ext.clone());
    }
    let transpiler: Option<Rc<dyn ModuleTranspiler>> =
      if self.enable_build_tools {
        Some(Rc::new(BabelTranspiler::new(resolver_config).await))
      } else {
        None
      };

    let mut main_module_path = std::env::current_dir()?.join(&self.module);
    if !main_module_path.exists() {
      let script = project_root.join("node_modules/.bin").join(&self.module);
      if script.exists() {
        main_module_path = script;
      }
    }
    if !main_module_path.exists() {
      bail!("missing script {:?}", self.module);
    }

    let egress_addr = self
      .egress_addr
      .as_ref()
      .map(|addr| addr.parse())
      .transpose()?;
    let mut runtime = IsolatedRuntime::new(RuntimeOptions {
      config: RuntimeConfig {
        egress_addr,
        process_args: vec![
          vec![
            "node".to_owned(),
            main_module_path.to_str().unwrap().to_owned(),
          ],
          self.args.clone(),
        ]
        .concat(),
        project_root: project_root.clone(),
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
        transpiler,
      ))),
      builtin_extensions: builtin_modules
        .iter()
        .map(|m| m.get_extension())
        .collect(),
      permissions: PermissionsContainer {
        fs: Some(FileSystemPermissions::allow_all("/".into())),
        net: Some(NetPermissions::allow_all()),
        timer: Some(TimerPermissions::allow_hrtime()),
      },
      ..Default::default()
    })?;

    runtime
      .execute_main_module(
        &Url::from_file_path(main_module_path.canonicalize().unwrap()).unwrap(),
      )
      .await?;
    runtime.run_event_loop().await
  }
}
