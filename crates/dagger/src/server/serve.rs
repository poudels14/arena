use anyhow::{bail, Result};
use clap::Parser;
use common::arena::ArenaConfig;
use common::deno::extensions::server::HttpServerConfig;
use common::deno::extensions::{BuiltinExtensions, BuiltinModule};
use deno_core::resolve_url_or_path;
use jsruntime::permissions::{FileSystemPermissions, PermissionsContainer};
use jsruntime::{IsolatedRuntime, RuntimeOptions};
use std::collections::HashSet;
use std::env::current_dir;
use std::path::Path;
use std::rc::Rc;
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
    let cwd_path = current_dir()?;
    let cwd = cwd_path.to_str().unwrap();
    let project_root = ArenaConfig::find_project_root()?;

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

    if self.enable_cloud_ext {
      builtin_modules
        .extend(vec![BuiltinModule::Custom(Rc::new(cloud::llm::extension))]);
    }

    let mut runtime = IsolatedRuntime::new(RuntimeOptions {
      project_root: Some(project_root.clone()),
      config: Some(ArenaConfig::load(&project_root).unwrap_or_default()),
      enable_console: true,
      transpile: self.transpile,
      builtin_extensions: BuiltinExtensions::with_modules(builtin_modules),
      permissions: PermissionsContainer {
        fs: Some(FileSystemPermissions {
          root: cwd_path.clone(),
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

    let entry_file = resolve_url_or_path(&self.entry, &cwd_path)?;

    let local = tokio::task::LocalSet::new();
    local
      .run_until(async move {
        runtime
          .execute_main_module_code(
            &Url::parse("file://main").unwrap(),
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
      })
      .await
  }
}
