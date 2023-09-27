use common::deno::extensions::server::HttpServerConfig;
use common::deno::extensions::{
  BuiltinExtension, BuiltinExtensions, BuiltinModule,
};
use common::deno::loader::BuiltInModuleLoader;
use common::resolve_from_root;
use deno_core::anyhow::Result;
use deno_core::ExtensionFileSource;
use deno_core::{anyhow, JsRuntime, OpState, RuntimeOptions};
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use url::Url;

struct Permissions;

impl deno_web::TimersPermission for Permissions {
  fn allow_hrtime(&mut self) -> bool {
    unreachable!("snapshotting!")
  }

  fn check_unstable(&self, _state: &OpState, _api_name: &'static str) {
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

fn get_basic_runtime() -> JsRuntime {
  deno_core::extension!(runtime,
    deps = [
      deno_webidl,
      deno_console,
      deno_url,
      deno_web,
      deno_fetch
    ],
    esm = [
      dir "../../js/arena-runtime/core/",
      "http.js"
    ],
    customizer = |ext: &mut deno_core::ExtensionBuilder| {
      ext.esm(vec![ExtensionFileSource {
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
      }]);
      ext.esm_entry_point("ext:runtime/main.js");
    }
  );

  let runtime = JsRuntime::new(RuntimeOptions {
    extensions: vec![
      // Note(sagar): deno_webidl, deno_url, deno_web need to be included for
      // timer (setTimeout, etc) to work
      deno_webidl::deno_webidl::init_js_only(),
      deno_console::deno_console::init_js_only(),
      deno_url::deno_url::init_ops_and_esm(),
      deno_web::deno_web::init_ops_and_esm::<Permissions>(
        deno_web::BlobStore::default(),
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
      runtime::init_ops_and_esm(),
    ],
    will_snapshot: true,
    module_loader: Some(Rc::new(BuiltInModuleLoader {})),
    ..Default::default()
  });
  runtime
}
