use common::deno::extensions::server::HttpServerConfig;
use common::deno::extensions::{
  BuiltinExtension, BuiltinExtensions, BuiltinModule,
};
use common::deno::loader::BuiltInModuleLoader;
use common::resolve_from_root;
use deno_core::anyhow::{self, Result};
use deno_core::{
  Extension, ExtensionFileSource, JsRuntimeForSnapshot, RuntimeOptions,
};
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use url::Url;

struct Permissions;

impl deno_web::TimersPermission for Permissions {
  fn allow_hrtime(&mut self) -> bool {
    unreachable!("snapshotting!")
  }
}

impl deno_fetch::FetchPermissions for Permissions {
  fn check_net_url(
    &mut self,
    _url: &Url,
    _api_name: &str,
  ) -> Result<(), anyhow::Error> {
    unreachable!("snapshotting!")
  }

  fn check_read(
    &mut self,
    _path: &Path,
    _api_name: &str,
  ) -> Result<(), anyhow::Error> {
    unreachable!("snapshotting!")
  }
}

pub fn main() {
  let o = PathBuf::from(env::var_os("OUT_DIR").unwrap());
  generate_prod_snapshot(&o.join("WORKSPACE_DQS_SNAPSHOT.bin"));
}

fn generate_prod_snapshot(path: &Path) {
  let mut runtime = get_basic_runtime();

  BuiltinExtensions::with_modules(vec![
    BuiltinModule::Node(Some(vec!["crypto"])),
    BuiltinModule::Postgres,
    BuiltinModule::Sqlite,
    // Note(sagar): load this here so that ESM modules are snapshotted
    // Even if TCP server is used here, we can use stream server during
    // runtime if needed
    BuiltinModule::HttpServer(HttpServerConfig::Tcp {
      address: "0.0.0.0".to_owned(),
      port: 0,
      serve_dir: None,
    }),
    BuiltinModule::Custom(Rc::new(|| cloud::extension(Default::default()))),
    BuiltinModule::Custom(Rc::new(|| BuiltinExtension {
      snapshot_modules: vec![
        (
          "@arena/dqs/widget-server",
          resolve_from_root!(
            "../../js/arena-runtime/dist/dqs/widget-server.js",
            true
          ),
        ),
        (
          "@arena/dqs/plugin/workflow/lib",
          resolve_from_root!(
            "../../js/arena-runtime/dist/dqs/plugin/workflow/lib.js",
            true
          ),
        ),
        (
          "@arena/dqs/utils",
          resolve_from_root!("../../js/arena-runtime/dist/dqs/utils.js", true),
        ),
        (
          "@arena/dqs/postgres",
          resolve_from_root!(
            "../../js/arena-runtime/dist/dqs/postgres.js",
            true
          ),
        ),
      ],
      ..Default::default()
    })),
  ])
  .load_snapshot_modules(&mut runtime)
  .unwrap();

  let snapshot: &[u8] = &*runtime.snapshot();
  std::fs::write(path, snapshot).unwrap();
}

fn get_basic_runtime() -> JsRuntimeForSnapshot {
  let runtime_ext = Extension {
    name: "runtime",
    deps: &[
      "deno_webidl",
      "deno_console",
      "deno_url",
      "deno_web",
      "deno_fetch",
    ],
    esm_files: vec![
      ExtensionFileSource {
        specifier: "ext:runtime/http.js",
        code: deno_core::ExtensionFileSourceCode::IncludedInBinary(
          include_str!("../../js/arena-runtime/core/http.js"),
        ),
      },
      ExtensionFileSource {
        specifier: "ext:runtime/setup.js",
        code: deno_core::ExtensionFileSourceCode::IncludedInBinary(
          include_str!("setup.js"),
        ),
      },
      ExtensionFileSource {
        specifier: "ext:runtime/main.js",
        code: deno_core::ExtensionFileSourceCode::IncludedInBinary(
          include_str!("main.js"),
        ),
      },
    ]
    .into(),
    esm_entry_point: Some("ext:runtime/main.js"),
    enabled: true,
    ..Default::default()
  };

  let runtime = JsRuntimeForSnapshot::new(RuntimeOptions {
    extensions: vec![
      // Note(sagar): deno_webidl, deno_url, deno_web need to be included for
      // timer (setTimeout, etc) to work
      deno_webidl::deno_webidl::init_ops_and_esm(),
      deno_console::deno_console::init_ops_and_esm(),
      deno_url::deno_url::init_ops_and_esm(),
      deno_web::deno_web::init_ops_and_esm::<Permissions>(
        Arc::new(deno_web::BlobStore::default()),
        Default::default(),
      ),
      deno_fetch::deno_fetch::init_ops_and_esm::<Permissions>(
        deno_fetch::Options {
          user_agent: "arena/snapshot".to_owned(),
          root_cert_store_provider: None,
          proxy: None,
          request_builder_hook: None,
          unsafely_ignore_certificate_errors: None,
          client_cert_chain_and_key: None,
          file_fetch_handler: Rc::new(deno_fetch::DefaultFileFetchHandler),
        },
      ),
      runtime_ext,
    ],
    module_loader: Some(Rc::new(BuiltInModuleLoader {})),
    ..Default::default()
  });
  runtime
}
