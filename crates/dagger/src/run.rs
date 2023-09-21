use anyhow::Result;
use clap::Parser;
use common::arena::ArenaConfig;
use common::deno::extensions::{BuiltinExtensions, BuiltinModule};
use deno_core::resolve_url_or_path;
use jsruntime::permissions::{
  FileSystemPermissions, NetPermissions, PermissionsContainer,
};
use jsruntime::{IsolatedRuntime, RuntimeOptions};
use std::collections::HashSet;
use std::rc::Rc;

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
    let mut builtin_modules = vec![
      BuiltinModule::Fs,
      BuiltinModule::Node(None),
      BuiltinModule::Env,
      BuiltinModule::Postgres,
      BuiltinModule::Sqlite,
    ];

    if self.enable_build_tools {
      builtin_modules.extend(vec![
        BuiltinModule::Transpiler,
        BuiltinModule::Resolver(project_root.clone()),
        BuiltinModule::Babel,
        BuiltinModule::Rollup,
      ])
    }

    if self.enable_cloud_ext {
      builtin_modules
        .extend(vec![BuiltinModule::Custom(Rc::new(cloud::extension))]);
    }

    let egress_addr = self
      .egress_addr
      .as_ref()
      .map(|addr| addr.parse())
      .transpose()?;
    let mut runtime = IsolatedRuntime::new(RuntimeOptions {
      project_root: Some(project_root.clone()),
      config: Some(ArenaConfig::find_in_path_hierachy().unwrap_or_default()),
      enable_console: true,
      transpile: !self.disable_transpile,
      builtin_extensions: BuiltinExtensions::with_modules(builtin_modules),
      permissions: PermissionsContainer {
        fs: Some(FileSystemPermissions {
          allowed_read_paths: HashSet::from_iter(vec![
            // allow all files
            "/".to_owned(),
          ]),
          allowed_write_paths: HashSet::from_iter(vec![
            // allow all files
            "/".to_owned(),
          ]),
          ..Default::default()
        }),
        net: Some(NetPermissions {
          restricted_urls: Some(HashSet::new()),
          ..Default::default()
        }),
        ..Default::default()
      },
      egress_addr,
      ..Default::default()
    })?;

    let main_module =
      resolve_url_or_path(&self.file, &std::env::current_dir()?)?;
    runtime.execute_main_module(&main_module).await?;
    runtime.run_event_loop().await
  }
}
