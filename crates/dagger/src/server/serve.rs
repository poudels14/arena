use std::path::Path;
use std::rc::Rc;

use anyhow::{bail, Result};
use clap::Parser;
use cloud::CloudExtensionProvider;
use common::required_env;
use runtime::buildtools::{transpiler::BabelTranspiler, FileModuleLoader};
use runtime::config::{ArenaConfig, RuntimeConfig};
use runtime::deno::core::resolve_url_or_path;
use runtime::extensions::server::HttpServerConfig;
use runtime::extensions::{
  BuiltinExtension, BuiltinExtensionProvider, BuiltinModule,
};
use runtime::permissions::{
  FileSystemPermissions, NetPermissions, PermissionsContainer,
};
use runtime::resolver::FilePathResolver;
use runtime::{IsolatedRuntime, RuntimeOptions};
use s3::creds::Credentials;
use url::Url;

use super::s3loader::{S3ModuleLoaderOptions, S3ModulerLoader};

#[derive(Parser, Debug)]
pub struct Command {
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

  /// Load main module from the given S3 bucket
  /// The following env variables must be set
  ///   - S3_ENDPOINT
  ///   - S3_ACCESS_KEY
  ///   - S3_SECRET_KEY
  #[arg(long)]
  s3bucket: Option<String>,

  /// A server entry file with request handler as default export
  pub entry: String,

  /// Directory to serve static files from
  #[arg(long)]
  pub static_dir: Option<String>,
}

impl Command {
  #[tracing::instrument(skip_all)]
  pub async fn execute(&self) -> Result<()> {
    let project_root = ArenaConfig::find_project_root()
      .unwrap_or_else(|_| std::env::current_dir().unwrap());
    let arena_config = ArenaConfig::load(&project_root).unwrap_or_default();

    let mut builtin_modules = vec![
      BuiltinModule::Env,
      BuiltinModule::Fs,
      BuiltinModule::Node(None),
      BuiltinModule::Postgres,
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

    let resolver_config = arena_config
      .server
      .javascript
      .as_ref()
      .and_then(|js| js.resolve.clone())
      .unwrap_or_default();
    if self.transpile {
      builtin_modules.extend(vec![
        BuiltinModule::Resolver(resolver_config.clone()),
        BuiltinModule::Transpiler,
        BuiltinModule::Babel,
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
      config: RuntimeConfig {
        project_root: project_root.clone(),
        ..Default::default()
      },
      enable_console: true,
      enable_arena_global: self.enable_cloud_ext,
      module_loader: if let Some(bucket) = &self.s3bucket {
        let endpoint = required_env!("S3_ENDPOINT");
        let access_key = required_env!("S3_ACCESS_KEY");
        let access_secret = required_env!("S3_ACCESS_SECRET");
        Some(Rc::new(S3ModulerLoader::new(
          self.entry.clone(),
          S3ModuleLoaderOptions {
            with_path_style: true,
            bucket: bucket.to_owned(),
            endpoint,
            credentials: Credentials {
              access_key: Some(access_key),
              secret_key: Some(access_secret),
              security_token: None,
              session_token: None,
              expiration: None,
            },
          },
        )))
      } else {
        Some(Rc::new(FileModuleLoader::new(
          Rc::new(FilePathResolver::new(
            project_root.clone(),
            arena_config
              .server
              .javascript
              .and_then(|j| j.resolve)
              .unwrap_or_default(),
          )),
          if self.transpile {
            Some(Rc::new(BabelTranspiler::new(resolver_config).await))
          } else {
            None
          },
        )))
      },
      builtin_extensions,
      permissions: PermissionsContainer {
        fs: Some(FileSystemPermissions::allow_all(project_root.clone())),
        net: Some(NetPermissions::allow_all()),
        ..Default::default()
      },
      ..Default::default()
    })?;

    // resolve entry file path if the entry file isn't being loaded from s3
    let entry_file = if self.s3bucket.is_none() {
      resolve_url_or_path(&self.entry, &project_root)?.to_string()
    } else {
      self.entry.clone()
    };

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
        true,
      )
      .await
  }
}
