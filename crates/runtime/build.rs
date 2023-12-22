#![allow(unused_doc_comments)]

use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Result;
use deno_core::snapshot_util::{create_snapshot, CreateSnapshotOptions};
use deno_core::ExtensionFileSource;
use deno_core::{anyhow, Extension, ExtensionFileSourceCode};
use url::Url;

macro_rules! load_during_snapshot {
  ($specifier:literal, $code:literal) => {
    ExtensionFileSource {
      specifier: $specifier,
      code: ExtensionFileSourceCode::LoadedFromFsDuringSnapshot($code),
    }
  };
}

pub fn main() {
  println!(
    "cargo:rustc-env=TARGET={}",
    std::env::var("TARGET").unwrap()
  );

  let o = PathBuf::from(env::var_os("OUT_DIR").unwrap());
  generate_prod_snapshot(o.join("BASE_RUNTIME_SNAPSHOT.bin"));
}

fn generate_prod_snapshot(snapshot_path: PathBuf) {
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
        specifier: "ext:runtime/0_setup.js",
        code: ExtensionFileSourceCode::LoadedFromFsDuringSnapshot(
          "./js/core/0_setup.js",
        ),
      },
      load_during_snapshot!("ext:runtime/http.js", "./js/core/http.js"),
      load_during_snapshot!("ext:runtime/1_arena.js", "./js/core/1_arena.js"),
      load_during_snapshot!(
        "ext:runtime/dummy-process.js",
        "./js/core/dummy-process.js"
      ),
      ExtensionFileSource {
        specifier: "ext:runtime/main.js",
        code: ExtensionFileSourceCode::LoadedFromFsDuringSnapshot("./main.js"),
      },
    ]
    .into(),
    esm_entry_point: Some("ext:runtime/main.js"),
    enabled: true,
    ..Default::default()
  };

  // Include only necessary modules in the base snapshot
  let extensions = vec![
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
  ];

  let output = create_snapshot(CreateSnapshotOptions {
    cargo_manifest_dir: env!("CARGO_MANIFEST_DIR"),
    snapshot_path,
    extensions,
    startup_snapshot: None,
    compression_cb: None,
    with_runtime_cb: None,
    skip_op_registration: false,
  });
  for path in output.files_loaded_during_snapshot {
    println!("cargo:rerun-if-changed={}", path.display());
  }
}

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
