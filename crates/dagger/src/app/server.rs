use std::path::PathBuf;
use std::rc::Rc;

use anyhow::Result;
use cloud::CloudExtensionProvider;
use runtime::buildtools::{transpiler::BabelTranspiler, FileModuleLoader};
use runtime::config::{ArenaConfig, RuntimeConfig};
use runtime::extensions::server::HttpServerConfig;
use runtime::extensions::{
  BuiltinExtension, BuiltinExtensionProvider, BuiltinModule,
};
use runtime::permissions::{
  FileSystemPermissions, NetPermissions, PermissionsContainer, TimerPermissions,
};
use runtime::resolver::FilePathResolver;
use runtime::{IsolatedRuntime, RuntimeOptions};
use url::Url;

pub(super) struct ServerOptions {
  pub root_dir: PathBuf,
  pub config: ArenaConfig,
  pub port: u16,
  pub address: String,
  pub transpile: bool,
}

pub(super) async fn start_js_server(
  options: ServerOptions,
  main_module: &str,
) -> Result<()> {
  let resolver_config = options
    .config
    .server
    .javascript
    .as_ref()
    .and_then(|js| js.resolve.clone())
    .unwrap_or_default();

  let mut builtin_modules = vec![
    BuiltinModule::Fs,
    BuiltinModule::Env,
    BuiltinModule::Node(None),
    BuiltinModule::Postgres,
    BuiltinModule::Resolver(resolver_config.clone()),
    BuiltinModule::Transpiler,
    BuiltinModule::HttpServer(HttpServerConfig::Tcp {
      address: options.address.clone(),
      port: options.port,
      serve_dir: if options.transpile {
        None
      } else {
        Some(options.root_dir.clone())
      },
    }),
  ];

  if options.transpile {
    builtin_modules.extend(vec![BuiltinModule::Babel])
  }

  let mut builtin_extensions: Vec<BuiltinExtension> =
    builtin_modules.iter().map(|m| m.get_extension()).collect();
  builtin_extensions.push(
    BuiltinModule::UsingProvider(Rc::new(CloudExtensionProvider::default()))
      .get_extension(),
  );

  let mut runtime = IsolatedRuntime::new(RuntimeOptions {
    config: RuntimeConfig {
      project_root: options.root_dir.clone(),
      ..Default::default()
    },
    enable_console: true,
    enable_arena_global: true,
    builtin_extensions,
    module_loader: Some(Rc::new(FileModuleLoader::new(
      Rc::new(FilePathResolver::new(
        options.root_dir.clone(),
        options
          .config
          .server
          .javascript
          .and_then(|j| j.resolve)
          .unwrap_or_default(),
      )),
      Some(Rc::new(BabelTranspiler::new(resolver_config).await)),
    ))),
    permissions: PermissionsContainer {
      fs: Some(FileSystemPermissions::allow_all("/".into())),
      net: Some(NetPermissions::allow_all()),
      timer: Some(TimerPermissions::allow_hrtime()),
    },
    ..Default::default()
  })?;

  runtime
    .execute_main_module_code(
      &Url::parse("file:///arena/app-server")?,
      main_module,
      true,
    )
    .await
}
