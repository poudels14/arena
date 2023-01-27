#![allow(unused_doc_comments)]
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

  generate_prod_snapshot(&o.join("RUNTIME_PROD_SNAPSHOT.bin"));
  // Note(sagar): when using snapshot for build tools, vm startup is 5x faster
  // than loading script during runtime, so using snapshot
  generate_builder_snapshot(&o.join("RUNTIME_BUILD_SNAPSHOT.bin"));
}

fn generate_prod_snapshot(path: &Path) {
  let runtime = get_basic_runtime();
  let snapshot: &[u8] = &*runtime.snapshot();
  std::fs::write(path, snapshot).unwrap();
}

// Note(sagar): this includes build tools like babel/transpiler/etc
fn generate_builder_snapshot(path: &Path) {
  {
    // TODO(sagar): can we transpile JS files before creating runtime in prod?

    // println!("cargo:rerun-if-changed=js/libs/src");
    // println!("cargo:rerun-if-changed=js/libs/package.json");
    // println!("cargo:rerun-if-changed=js/libs/pnpm-lock.yaml");
    // println!("cargo:rerun-if-changed=js/libs/rollup.config.js");
    // let js_packages = env::current_dir().unwrap().to_str().unwrap().to_string() + "/js/libs";
    // Command::new("pnpm")
    //   .args(["--dir", &js_packages, "i"])
    //   .output()
    //   .expect("failed to build js packages");

    // Command::new("pnpm")
    //   .args(["--dir", &js_packages, "build"])
    //   .output()
    //   .expect("failed to build js packages");

    // // Note(sagar): it seems like rollup output files aren't ready as soon as
    // // the above commands are finished. so, wait for a bit so that the files are
    // // written to filesystem
    // std::thread::sleep(std::time::Duration::from_millis(200));
  }

  let mut runtime = get_basic_runtime();
  runtime
    .execute_script("<arena/babel>", include_str!("./js/libs/dist/babel.js"))
    .unwrap();
  let snapshot: &[u8] = &*runtime.snapshot();
  std::fs::write(path, snapshot).unwrap();
}

fn get_basic_runtime() -> JsRuntime {
  let mut runtime = JsRuntime::new(RuntimeOptions {
    extensions_with_js: vec![
      /**
       * Note(sagar): deno_webidl, deno_url, deno_web need to be included for
       * timer (setTimeout, etc) to work
       */
      deno_webidl::init(),
      deno_console::init(),
      deno_url::init(),
      deno_web::init::<Permissions>(
        deno_web::BlobStore::default(),
        Default::default(),
      ),
      deno_fetch::init::<Permissions>(deno_fetch::Options {
        user_agent: "arena/snapshot".to_owned(),
        root_cert_store: None,
        proxy: None,
        request_builder_hook: None,
        unsafely_ignore_certificate_errors: None,
        client_cert_chain_and_key: None,
        file_fetch_handler: Rc::new(deno_fetch::DefaultFileFetchHandler),
      }),
    ],
    will_snapshot: true,
    ..Default::default()
  });

  runtime
    .execute_script(
      "<arena/core/process>",
      include_str!("./js/core/dummy-process.js"),
    )
    .unwrap();

  runtime
}
