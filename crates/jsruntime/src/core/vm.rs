use super::loaders;
use crate::permissions::Permissions;
use anyhow::{anyhow, Result};
use deno_core::{v8, JsRealm};
use deno_core::{Extension, JsRuntime, Snapshot};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use url::Url;

pub static RUNTIME_PROD_SNAPSHOT: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/RUNTIME_PROD_SNAPSHOT.bin"));

pub static RUNTIME_BUILD_SNAPSHOT: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/RUNTIME_BUILD_SNAPSHOT.bin"));

#[derive(Default, Clone)]
pub struct RuntimeConfig {
  /// enable build tools like babel, babel plugins, etc
  pub enable_build_tools: bool,

  /// whether to auto-transpile the code when loading
  pub transpile: bool,

  pub disable_module_loader: bool,

  /// Name of the HTTP user_agent
  pub user_agent: String,

  pub permissions: Permissions,

  pub enable_console: bool,
}

pub struct IsolatedRuntime {
  pub config: RuntimeConfig,
  pub runtime: Arc<Mutex<JsRuntime>>,
}

impl IsolatedRuntime {
  pub fn new(config: RuntimeConfig) -> IsolatedRuntime {
    let js_runtime =
      Arc::new(Mutex::new(JsRuntime::new(deno_core::RuntimeOptions {
        // TODO(sagar): remove build snapshot from deployed app runner to save memory
        startup_snapshot: Some(if config.enable_build_tools {
          Snapshot::Static(RUNTIME_BUILD_SNAPSHOT)
        } else {
          Snapshot::Static(RUNTIME_PROD_SNAPSHOT)
        }),
        module_loader: if config.disable_module_loader {
          None
        } else {
          Some(std::rc::Rc::new(loaders::FsModuleLoader::new(
            &loaders::ModuleLoaderOption {
              transpile: config.transpile,
            },
          )))
        },
        // Note(sagar) Since the following extensions were snapshotted, pass them
        // as `extensions` instead of `extensions_with_js`; only rust bindings are
        // necessary since JS is already loaded
        extensions: vec![
          deno_webidl::init(),
          deno_console::init(),
          deno_url::init(),
          deno_web::init::<Permissions>(
            deno_web::BlobStore::default(),
            Default::default(),
          ),
          deno_fetch::init::<Permissions>(deno_fetch::Options {
            user_agent: "arena/server".to_string(),
            root_cert_store: None,
            proxy: None,
            request_builder_hook: None,
            unsafely_ignore_certificate_errors: None,
            client_cert_chain_and_key: None,
            file_fetch_handler: Rc::new(deno_fetch::DefaultFileFetchHandler),
          }),
          Extension::builder("permission_setter")
            .state(move |state| {
              state.put::<Permissions>(Permissions {
                timer: None,
                net: None,
              });
              Ok(())
            })
            .build(),
          // ext::error_ops::init(),
          // ext::response_ops::init(),
          // ext::postgres_ops::init(),
        ],
        extensions_with_js: vec![Extension::builder("<arena/init>")
          .js(Self::get_js_extensions(&config))
          .build()],
        ..Default::default()
      })));

    let runtime = IsolatedRuntime {
      config: config.clone(),
      runtime: js_runtime,
    };

    runtime
  }

  pub async fn execute_main_module(&mut self, url: &Url) -> Result<()> {
    let mut runtime = self
      .runtime
      .lock()
      .map_err(|e| anyhow!("failed to get lock to runtime: {:?}", e))?;
    let mod_id = runtime.load_main_module(url, None).await?;
    let receiver = runtime.mod_evaluate(mod_id);

    runtime.run_event_loop(false).await?;
    receiver.await?
  }

  pub async fn run_event_loop(&mut self) -> Result<()> {
    let mut runtime = self
      .runtime
      .lock()
      .map_err(|e| anyhow!("failed to get lock to runtime: {:?}", e))?;
    runtime.run_event_loop(false).await
  }

  #[allow(dead_code)]
  pub fn execute_script(
    &mut self,
    name: &str,
    code: &str,
  ) -> Result<v8::Global<v8::Value>> {
    let mut runtime = self
      .runtime
      .lock()
      .map_err(|e| anyhow!("failed to get lock to runtime: {:?}", e))?;
    runtime
      .execute_script(name, code)
      .map_err(|e| anyhow!("V8 error: {:?}", e))
  }

  /// Initializes a Javascript function in the context of this runtime
  #[allow(dead_code)]
  pub fn init_js_function(
    &mut self,
    code: &str,
    realm: Option<JsRealm>,
  ) -> Result<super::function::Function> {
    super::function::Function::new(self.runtime.clone(), code, realm)
  }

  fn get_js_extensions(
    config: &RuntimeConfig,
  ) -> Vec<(&'static str, &'static str)> {
    let mut vec = Vec::new();
    vec.push((
      "<arena/init>",
      r#"
      Deno.core.initializeAsyncOps();
      const { setTimeout, clearTimeout, setInterval, clearInterval, handleTimerMacrotask } = globalThis.__bootstrap.timers;
      Deno.core.setMacrotaskCallback(handleTimerMacrotask);
      const { Request, Response } = globalThis.__bootstrap.fetch;
      const { ReadableStream } = globalThis.__bootstrap.streams;
      "#,
    ));

    if config.enable_console {
      vec.push((
        "<arena/console>",
        r#"
        ((globalThis) => {
          globalThis.console = new globalThis.__bootstrap.console.Console(Deno.core.print);
        })(globalThis);
        "#,
      ));
    }

    vec
  }
}
