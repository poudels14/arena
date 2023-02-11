use super::loaders::{self, ModuleLoaderConfig};
use crate::permissions::Permissions;
use anyhow::{anyhow, Result};
use deno_core::{v8, JsRealm};
use deno_core::{Extension, JsRuntime, Snapshot};
use log::error;
use std::cell::RefCell;
use std::rc::Rc;
use url::Url;

pub static RUNTIME_PROD_SNAPSHOT: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/RUNTIME_PROD_SNAPSHOT.bin"));

pub static RUNTIME_BUILD_SNAPSHOT: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/RUNTIME_BUILD_SNAPSHOT.bin"));

#[derive(Default)]
pub struct RuntimeConfig {
  /// Name of the HTTP user_agent
  pub user_agent: String,

  pub permissions: Permissions,

  pub enable_console: bool,

  /// Additional extensions to add to the runtime
  pub extensions: Vec<Extension>,

  /// Heap limit tuple: (initial size, max hard limit) in bytes
  pub heap_limits: Option<(usize, usize)>,

  /// enable build tools like babel, babel plugins, etc
  pub enable_build_tools: bool,

  /// whether to auto-transpile the code when loading
  pub transpile: bool,

  pub disable_module_loader: bool,

  /// Config used by module loader
  pub module_loader_config: Option<ModuleLoaderConfig>,
}

pub struct IsolatedRuntime {
  pub config: RuntimeConfig,
  pub runtime: Rc<RefCell<JsRuntime>>,
}

impl IsolatedRuntime {
  pub fn new(mut config: RuntimeConfig) -> IsolatedRuntime {
    let mut extensions_with_js = Self::get_js_extensions(&mut config);
    // Note(sagar): take extensions out of the config and set it to empty
    // vec![] so that config can be stored without having Send trait
    if config.extensions.len() > 0 {
      let exts = config.extensions;
      extensions_with_js.extend(exts);
      config.extensions = vec![];
    }

    let create_params = config.heap_limits.map(|(initial, max)| {
      v8::Isolate::create_params().heap_limits(initial, max)
    });

    let mut js_runtime = JsRuntime::new(deno_core::RuntimeOptions {
      // TODO(sagar): remove build snapshot from deployed app runner to save memory
      startup_snapshot: Some(if config.enable_build_tools {
        Snapshot::Static(RUNTIME_BUILD_SNAPSHOT)
      } else {
        Snapshot::Static(RUNTIME_PROD_SNAPSHOT)
      }),
      create_params,
      module_loader: if config.disable_module_loader {
        None
      } else {
        Some(Rc::new(loaders::FsModuleLoader::new(
          loaders::ModuleLoaderOption {
            transpile: config.transpile,
            config: config.module_loader_config.take().unwrap(),
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
        Extension::builder("<arena/core/permissions/setter>")
          .state(move |state| {
            state.put::<Permissions>(Permissions {
              timer: None,
              net: None,
            });
            Ok(())
          })
          .build(),
      ],
      extensions_with_js,
      ..Default::default()
    });

    // Note(sagar): if the heap limits are set, terminate the runtime manually
    if config.heap_limits.is_some() {
      let cb_handle = js_runtime.v8_isolate().thread_safe_handle();
      js_runtime.add_near_heap_limit_callback(
        move |current_limit, _initial_limit| {
          error!("Terminating V8 due to memory limit");
          cb_handle.terminate_execution();
          current_limit
        },
      );
    }

    let runtime = IsolatedRuntime {
      config,
      runtime: Rc::new(RefCell::new(js_runtime)),
    };

    runtime
  }

  pub async fn execute_main_module(&mut self, url: &Url) -> Result<()> {
    let mut runtime = self.runtime.borrow_mut();
    let mod_id = runtime.load_main_module(url, None).await?;
    let receiver = runtime.mod_evaluate(mod_id);

    runtime.run_event_loop(false).await?;
    receiver.await?
  }

  #[allow(dead_code)]
  pub async fn execute_main_module_code(
    &mut self,
    url: &Url,
    code: &str,
  ) -> Result<()> {
    let mut runtime = self.runtime.borrow_mut();
    let mod_id = runtime.load_main_module(url, Some(code.to_owned())).await?;
    let receiver = runtime.mod_evaluate(mod_id);

    runtime.run_event_loop(false).await?;
    receiver.await?
  }

  pub async fn run_event_loop(&mut self) -> Result<()> {
    let mut runtime = self.runtime.borrow_mut();
    runtime.run_event_loop(false).await
  }

  #[allow(dead_code)]
  pub fn execute_script(
    &mut self,
    name: &str,
    code: &str,
  ) -> Result<v8::Global<v8::Value>> {
    let mut runtime = self.runtime.borrow_mut();
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

  fn get_js_extensions(config: &RuntimeConfig) -> Vec<Extension> {
    let mut extensions = Vec::new();

    let mut js_files = Vec::new();
    js_files.push(("<arena/init>", include_str!("../../js/core/setup.js")));
    js_files.push(("<arena/arena>", include_str!("../../js/core/0_arena.js")));
    if config.enable_console {
      js_files.push((
        "<arena/console>",
        r#"
        ((globalThis) => {
          globalThis.console = new globalThis.__bootstrap.console.Console(Deno.core.print);
        })(globalThis);
        "#,
      ));
    }

    extensions.push(Extension::builder("<arena/init>").js(js_files).build());

    extensions
  }
}
