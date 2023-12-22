use std::path::PathBuf;
use std::rc::Rc;

use anyhow::Result;
use runtime::extensions::server::HttpServerConfig;
use runtime::extensions::{BuiltinExtensionProvider, BuiltinModule};
use runtime::permissions::{
  FileSystemPermissions, NetPermissions, PermissionsContainer,
};
use runtime::{
  FileModuleLoader, FilePathResolver, IsolatedRuntime, ModuleLoaderOption,
  RuntimeOptions,
};
use url::Url;

pub(super) struct ServerOptions {
  pub root_dir: PathBuf,
  pub port: u16,
  pub address: String,
  pub transpile: bool,
}

pub(super) async fn start_server(
  options: ServerOptions,
  main_module: &str,
) -> Result<()> {
  let mut builtin_modules = vec![
    BuiltinModule::Fs,
    BuiltinModule::Env,
    BuiltinModule::Node(None),
    BuiltinModule::Postgres,
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
    // TODO
    // let cloud_ext =
    // BuiltinModule::UsingProvider(Rc::new(CloudExtensionProvider {
    //   publisher: None,
    // }));
    builtin_modules.extend(vec![
      BuiltinModule::Sqlite,
      BuiltinModule::Resolver(options.root_dir.clone()),
      BuiltinModule::Transpiler,
      BuiltinModule::FileRouter,
    ])
  }

  let mut runtime = IsolatedRuntime::new(RuntimeOptions {
    enable_console: true,
    enable_arena_global: true,
    builtin_extensions: builtin_modules
      .iter()
      .map(|m| m.get_extension())
      .collect(),
    module_loader: Some(Rc::new(FileModuleLoader::new(ModuleLoaderOption {
      transpile: true,
      resolver: Rc::new(FilePathResolver::new(
        options.root_dir.clone(),
        Default::default(),
      )),
    }))),
    permissions: PermissionsContainer {
      fs: Some(FileSystemPermissions::allow_all(options.root_dir)),
      net: Some(NetPermissions::allow_all()),
      ..Default::default()
    },
    ..Default::default()
  })?;

  runtime
    .execute_main_module_code(
      &Url::parse("file:///arena/app-server")?,
      main_module,
    )
    .await?;
  runtime.run_event_loop().await
}
