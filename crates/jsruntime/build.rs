#![allow(unused_doc_comments)]
use common::deno::extensions;
use deno_core::anyhow::{bail, Error, Result};
use deno_core::{
  anyhow, JsRuntime, ModuleLoader, ModuleSourceFuture, ModuleSpecifier,
  OpState, ResolutionKind, RuntimeOptions,
};
use deno_core::{ExtensionFileSource, ExtensionFileSourceCode};
use futures::FutureExt;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;
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
}

fn generate_prod_snapshot(path: &Path) {
  let mut runtime = get_basic_runtime();

  let exts: Vec<ExtensionFileSource> = vec![
    extensions::node::get_modules_for_snapshotting(),
    extensions::buildtools::get_modules_for_snapshotting(),
  ]
  .iter()
  .flatten()
  .map(|s| s.clone())
  .collect();

  for module in exts.iter() {
    futures::executor::block_on(async {
      let mod_id = runtime
        .load_side_module(
          &Url::parse(&format!("builtin:///{}", module.specifier))?,
          Some(module.code.load()?),
        )
        .await?;
      let receiver = runtime.mod_evaluate(mod_id);
      runtime.run_event_loop(false).await?;
      receiver.await?
    })
    .unwrap();
  }

  let snapshot: &[u8] = &*runtime.snapshot();
  std::fs::write(path, snapshot).unwrap();
}

fn get_basic_runtime() -> JsRuntime {
  let core_extension = deno_core::Extension::builder("core")
    .esm(vec![
      ExtensionFileSource {
        specifier: "setup".to_string(),
        code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
          "../../js/arena-runtime/core/0_setup.js"
        )),
      },
      ExtensionFileSource {
        specifier: "arena/setup".to_string(),
        code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
          "../../js/arena-runtime/core/1_arena.js"
        )),
      },
      ExtensionFileSource {
        specifier: "arena/process".to_string(),
        code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
          "../../js/arena-runtime/core/dummy-process.js"
        )),
      },
    ])
    .js(vec![ExtensionFileSource {
      specifier: "init".to_string(),
      code: ExtensionFileSourceCode::IncludedInBinary(
        r#"
          Arena.core = Deno.core;
        "#,
      ),
    }])
    .build();

  let runtime = JsRuntime::new(RuntimeOptions {
    extensions: vec![
      /**
       * Note(sagar): deno_webidl, deno_url, deno_web need to be included for
       * timer (setTimeout, etc) to work
       */
      deno_webidl::init_esm(),
      deno_console::init_esm(),
      deno_url::init_ops_and_esm(),
      deno_web::init_ops_and_esm::<Permissions>(
        deno_web::BlobStore::default(),
        Default::default(),
      ),
      deno_crypto::init_ops_and_esm(None),
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
      extensions::fs::init_js_and_ops(),
    ],
    will_snapshot: true,
    module_loader: Some(Rc::new(BuiltInModuleLoader {})),
    ..Default::default()
  });

  runtime
}

struct BuiltInModuleLoader {}

impl ModuleLoader for BuiltInModuleLoader {
  fn resolve(
    &self,
    specifier: &str,
    _referrer: &str,
    _kind: ResolutionKind,
  ) -> Result<ModuleSpecifier, Error> {
    // Note(sagar): since all modules during build are builtin modules,
    // add url schema `builtin:///` prefix
    let specifier = match specifier.starts_with("builtin:///") {
      true => specifier.to_string(),
      false => format!("builtin:///{}", specifier),
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
    maybe_referrer: Option<ModuleSpecifier>,
    _is_dynamic: bool,
  ) -> Pin<Box<ModuleSourceFuture>> {
    let specifier = module_specifier.clone();
    let referrer = maybe_referrer.clone();
    async move {
      bail!(
        "Module loading not supported: specifier = {:?}, referrer = {:?}",
        specifier.as_str(),
        referrer.as_ref().map(|r| r.as_str())
      );
    }
    .boxed_local()
  }
}
