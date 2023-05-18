use common::deno::extensions::server::HttpServerConfig;
use common::deno::extensions::{
  BuiltinExtension, BuiltinExtensions, BuiltinModule,
};
use common::deno::loader::BuiltInModuleLoader;
use common::resolve_from_root;
use deno_core::anyhow::Result;
use deno_core::{anyhow, JsRuntime, OpState, RuntimeOptions};
use deno_core::{ExtensionFileSource, ExtensionFileSourceCode};
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
    // Note(sagar): load this here so that ESM modules are snapshotted
    // Even if TCP server is used here, we can use stream server during
    // runtime if needed
    BuiltinModule::Postgres,
    BuiltinModule::HttpServer(HttpServerConfig::Tcp("0.0.0.0".to_owned(), 0)),
    BuiltinModule::Custom(|| BuiltinExtension {
      snapshot_modules: vec![
        // Note(sagar): load this under @arena/dqs/router instead of
        // @arena/functions/router since we dont want user code to be able
        // to load this module and all @arena/functions/... are accessible
        // by user code
        (
          "@arena/dqs/router",
          resolve_from_root!(
            "../../js/arena-runtime/dist/functions/router.js",
            true
          ),
        ),
        dqs_function!("sql"),
        dqs_function!("sql/postgres"),
      ],
      ..Default::default()
    }),
  ])
  .load_snapshot_modules(&mut runtime)
  .unwrap();

  let snapshot: &[u8] = &*runtime.snapshot();
  std::fs::write(path, snapshot).unwrap();
}

fn get_basic_runtime() -> JsRuntime {
  let core_extension = deno_core::Extension::builder("core")
    .esm(vec![
      ExtensionFileSource {
        specifier: "init".to_string(),
        code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
          "./setup.js"
        )),
      },
      ExtensionFileSource {
        specifier: "http".to_string(),
        code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
          "../../js/arena-runtime/core/http.js"
        )),
      },
    ])
    .build();

  let runtime = JsRuntime::new(RuntimeOptions {
    extensions: vec![
      // Note(sagar): deno_webidl, deno_url, deno_web need to be included for
      // timer (setTimeout, etc) to work
      deno_webidl::init_esm(),
      deno_console::init_esm(),
      deno_url::init_ops_and_esm(),
      deno_web::init_ops_and_esm::<Permissions>(
        deno_web::BlobStore::default(),
        Default::default(),
      ),
      deno_fetch::init_ops_and_esm::<Permissions>(deno_fetch::Options {
        user_agent: "arena/snapshot".to_owned(),
        root_cert_store: None,
        proxy: None,
        request_builder_hook: None,
        unsafely_ignore_certificate_errors: None,
        client_cert_chain_and_key: None,
        file_fetch_handler: Rc::new(deno_fetch::DefaultFileFetchHandler),
      }),
      core_extension,
    ],
    will_snapshot: true,
    module_loader: Some(Rc::new(BuiltInModuleLoader {})),
    ..Default::default()
  });

  runtime
}

#[macro_export]
macro_rules! dqs_function {
  ($a:literal) => {{
    (
      concat!("@arena/functions/", $a),
      resolve_from_root!(
        concat!("../../js/arena-runtime/dist/functions/", $a, ".js"),
        true
      ),
    )
  }};
}
