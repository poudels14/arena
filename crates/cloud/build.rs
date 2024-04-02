use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::{bail, Result};
use deno_ast::ModuleSpecifier;
use deno_core::{
  Extension, ExtensionFileSource, JsRuntimeForSnapshot, RuntimeOptions,
};
use deno_core::{ModuleLoader, ModuleSourceFuture, ResolutionKind};
use futures::FutureExt;
use runtime::extensions::server::HttpServerConfig;
use runtime::extensions::BuiltinExtensionProvider;
use runtime::extensions::SourceCode;
use runtime::extensions::{BuiltinExtension, BuiltinExtensions, BuiltinModule};
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

macro_rules! include_from_project_root {
  ($file:literal) => {{
    println!(
      "cargo:rerun-if-changed={}",
      concat!(env!("CARGO_MANIFEST_DIR"), "/", $file)
    );
    SourceCode::Preserved(include_str!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/",
      $file
    )))
  }};
}

pub fn main() {
  println!(
    "cargo:rustc-env=CARGO_CFG_TARGET_OS={}",
    env::var("CARGO_CFG_TARGET_OS").unwrap()
  );
  println!(
    "cargo:rustc-env=CARGO_CFG_TARGET_ARCH={}",
    env::var("CARGO_CFG_TARGET_ARCH").unwrap()
  );
  let o = PathBuf::from(env::var_os("OUT_DIR").unwrap());
  generate_prod_snapshot(&o.join("WORKSPACE_DQS_SNAPSHOT.bin"));
}

fn generate_prod_snapshot(path: &Path) {
  let mut runtime = get_basic_runtime();

  let builtin_extensions = vec![
    BuiltinModule::Node(Some(vec!["crypto"])),
    BuiltinModule::Postgres,
    // Note(sagar): load this here so that ESM modules are snapshotted
    // Even if TCP server is used here, we can use stream server during
    // runtime if needed
    BuiltinModule::HttpServer(HttpServerConfig::Tcp {
      address: "0.0.0.0".to_owned(),
      port: 0,
      serve_dir: None,
    }),
    BuiltinModule::Custom(Rc::new(|| {
      BuiltinExtension::new(
        None,
        vec![(
          "@arena/dqs/postgres",
          include_from_project_root!("../../js/runtime/dist/dqs/postgres.js"),
        )],
      )
    })),
  ]
  .iter()
  .map(|m| m.get_extension())
  .collect();

  BuiltinExtensions::load_extensions(&builtin_extensions, &mut runtime)
    .expect("Error loading builtin extensions");

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
          include_str!("../runtime/js/core/http.js"),
        ),
      },
      ExtensionFileSource {
        specifier: "ext:runtime/setup.js",
        code: deno_core::ExtensionFileSourceCode::IncludedInBinary(
          include_str!("./src/dqs_runtime/setup.js"),
        ),
      },
      ExtensionFileSource {
        specifier: "ext:runtime/main.js",
        code: deno_core::ExtensionFileSourceCode::IncludedInBinary(
          include_str!("./src/dqs_runtime/main.js"),
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

pub struct BuiltInModuleLoader {}

impl ModuleLoader for BuiltInModuleLoader {
  fn resolve(
    &self,
    specifier: &str,
    _referrer: &str,
    _kind: ResolutionKind,
  ) -> Result<ModuleSpecifier> {
    let specifier = specifier.strip_prefix("node:").unwrap_or(specifier);
    // Note(sagar): since all modules during build are builtin modules,
    // add url schema `builtin://` prefix
    let specifier = match specifier.starts_with("builtin://") {
      true => specifier.to_string(),
      false => format!("builtin://{}", specifier),
    };

    match Url::parse(&specifier) {
      Ok(url) => Ok(url),
      _ => {
        bail!("Failed to resolve specifier: {:?}", specifier);
      }
    }
  }

  fn load(
    &self,
    module_specifier: &ModuleSpecifier,
    maybe_referrer: Option<&ModuleSpecifier>,
    _is_dynamic: bool,
  ) -> Pin<Box<ModuleSourceFuture>> {
    let specifier = module_specifier.clone();
    let referrer = maybe_referrer.as_ref().map(|r| r.to_string());
    async move {
      bail!(
        "Module loading not supported: specifier = {:?}, referrer = {:?}",
        specifier.as_str(),
        referrer
      );
    }
    .boxed_local()
  }
}
