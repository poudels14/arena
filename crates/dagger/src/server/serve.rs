use std::env::current_dir;
use std::path::Path;
use std::rc::Rc;

use anyhow::{bail, Result};
use clap::Parser;
use cloud::CloudExtensionProvider;
use runtime::buildtools::{FileModuleLoader, FilePathResolver};
use runtime::config::ArenaConfig;
use runtime::deno::core::resolve_url_or_path;
use runtime::extensions::server::HttpServerConfig;
use runtime::extensions::{
  BuiltinExtension, BuiltinExtensionProvider, BuiltinModule,
};
use runtime::permissions::{
  FileSystemPermissions, NetPermissions, PermissionsContainer,
};
use runtime::{IsolatedRuntime, RuntimeOptions};
use url::Url;

#[derive(Parser, Debug)]
pub struct Command {
  /// A server entry file with request handler as default export
  pub entry: String,

  /// Server host; default
  #[arg(long, default_value_t = String::from("0.0.0.0"))]
  pub host: String,

  /// Server port
  #[arg(short, long, default_value_t = 8000)]
  pub port: u16,

  /// Whether to transpile files when importing; default false
  #[clap(short, long, action, default_value_t = false)]
  pub transpile: bool,

  /// Enable @arena/cloud extension
  #[arg(long)]
  enable_cloud_ext: bool,

  /// Directory to serve static files from
  #[arg(long)]
  pub static_dir: Option<String>,
}

impl Command {
  #[tracing::instrument(skip_all)]
  pub async fn execute(&self) -> Result<()> {
    let cwd = current_dir()?;
    let project_root = ArenaConfig::find_project_root()?;
    let arena_config = ArenaConfig::load(&project_root)?;

    let mut builtin_modules = vec![
      BuiltinModule::Env,
      BuiltinModule::Fs,
      BuiltinModule::Node(None),
      BuiltinModule::Postgres,
      BuiltinModule::Sqlite,
      BuiltinModule::HttpServer(HttpServerConfig::Tcp {
        address: self.host.clone(),
        port: self.port,
        serve_dir: self
          .static_dir
          .clone()
          .map(|d| Path::new(&d).to_path_buf())
          .map(|dir| {
            if !dir.exists() {
              bail!("Invalid static directory")
            } else {
              Ok(dir)
            }
          })
          .transpose()?,
      }),
    ];

    if self.transpile {
      builtin_modules.extend(vec![
        BuiltinModule::Resolver(project_root.clone()),
        BuiltinModule::Transpiler,
        BuiltinModule::Babel,
        BuiltinModule::Rollup,
      ]);
    }

    let mut builtin_extensions: Vec<BuiltinExtension> =
      builtin_modules.iter().map(|m| m.get_extension()).collect();

    if self.enable_cloud_ext {
      builtin_extensions.push(
        BuiltinModule::UsingProvider(
          Rc::new(CloudExtensionProvider::default()),
        )
        .get_extension(),
      );
    }

    let mut runtime = IsolatedRuntime::new(RuntimeOptions {
      enable_console: true,
      enable_arena_global: self.enable_cloud_ext,
      module_loader: Some(Rc::new(FileModuleLoader::new(
        Rc::new(FilePathResolver::new(
          cwd.clone(),
          arena_config
            .server
            .javascript
            .and_then(|j| j.resolve)
            .unwrap_or_default(),
        )),
        None,
      ))),
      builtin_extensions,
      permissions: PermissionsContainer {
        fs: Some(FileSystemPermissions::allow_all(cwd.clone())),
        net: Some(NetPermissions::allow_all()),
        ..Default::default()
      },
      ..Default::default()
    })?;

    let entry_file = resolve_url_or_path(&self.entry, &cwd)?;
    runtime
      .execute_main_module_code(
        &Url::parse("file:///main").unwrap(),
        &format!(
          r#"
          import {{ serve }} from "@arena/runtime/server";
          import handler from "{0}";
          serve(handler);
          "#,
          entry_file
        ),
      )
      .await?;

    runtime.run_event_loop().await
  }
}
