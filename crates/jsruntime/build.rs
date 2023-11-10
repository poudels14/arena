#![allow(unused_doc_comments)]
use common::deno::extensions::BuiltinExtensions;
use common::deno::loader::BuiltInModuleLoader;
use deno_core::anyhow::Result;
use deno_core::{anyhow, Extension, ExtensionFileSourceCode, RuntimeOptions};
use deno_core::{ExtensionFileSource, JsRuntimeForSnapshot};
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
  generate_prod_snapshot(&o.join("RUNTIME_PROD_SNAPSHOT.bin"));
}

fn generate_prod_snapshot(path: &Path) {
  let mut runtime = get_basic_runtime();

  BuiltinExtensions::with_all_modules()
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
      file_source(
        "ext:runtime/0_setup.js",
        include_str!("../../js/arena-runtime/core/0_setup.js"),
      ),
      file_source(
        "ext:runtime/1_arena.js",
        include_str!("../../js/arena-runtime/core/1_arena.js"),
      ),
      file_source(
        "ext:runtime/dummy-process.js",
        include_str!("../../js/arena-runtime/core/dummy-process.js",),
      ),
      file_source(
        "ext:runtime/http.js",
        include_str!("../../js/arena-runtime/core/http.js"),
      ),
      file_source("ext:runtime/main.js", include_str!("./main.js")),
    ]
    .into(),
    esm_entry_point: Some("ext:runtime/main.js"),
    enabled: true,
    ..Default::default()
  };

  let runtime = JsRuntimeForSnapshot::new(RuntimeOptions {
    extensions: vec![
      /**
       * Note(sagar): deno_webidl, deno_url, deno_web need to be included for
       * timer (setTimeout, etc) to work
       */
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

fn file_source(
  specifier: &'static str,
  code: &'static str,
) -> ExtensionFileSource {
  ExtensionFileSource {
    specifier,
    code: ExtensionFileSourceCode::IncludedInBinary(code),
  }
}
