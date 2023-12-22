use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use deno_core::{
  v8, ExtensionFileSource, ExtensionFileSourceCode, FastString, ModuleLoader,
};
use deno_core::{Extension, JsRuntime, Snapshot};
use deno_fetch::CreateHttpClientOptions;
use derivative::Derivative;
use tracing::error;
use tracing::trace;
use url::Url;

use super::moduleloader::DefaultModuleLoader;
use crate::config::RuntimeConfig;
use crate::extensions::{BuiltinExtension, BuiltinExtensions};
use crate::permissions::PermissionsContainer;
use crate::utils;

pub static BASE_RUNTIME_SNAPSHOT: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/BASE_RUNTIME_SNAPSHOT.bin"));

macro_rules! include_in_binary {
  ($specifier:literal, $code:tt) => {
    include_in_binary!($specifier, $code, true)
  };
  ($specifier:literal, $code:tt, $condition:expr) => {
    ExtensionFileSource {
      specifier: $specifier,
      code: ExtensionFileSourceCode::IncludedInBinary(if $condition {
        $code
      } else {
        ""
      }),
    }
  };
}

#[derive(Derivative)]
#[derivative(Default)]
pub struct RuntimeOptions {
  pub enable_console: bool,

  /// Name of the HTTP user_agent
  pub user_agent: Option<String>,

  pub permissions: PermissionsContainer,

  /// Heap limit tuple: (initial size, max hard limit) in bytes
  pub heap_limits: Option<(usize, usize)>,

  /// Additional extensions to add to the runtime
  pub extensions: Vec<Extension>,

  pub builtin_extensions: Vec<BuiltinExtension>,

  pub module_loader: Option<Rc<dyn ModuleLoader>>,

  pub config: RuntimeConfig,

  /// Arena config to be used for the runtime
  /// If None is passed, package.json is checked
  /// in the current directory as well as up the directory tree
  // #[derivative(Default(value = "Option::None"))]
  // pub config: Option<ArenaConfig>,

  /// If set to true, `globalThis.Deno` and `globalThis.Arena` will be
  /// left intact. Else, Deno will be removed from globalThis and Arena
  /// will only have few required fields
  pub enable_arena_global: bool,
}

pub struct IsolatedRuntime {
  pub runtime: Rc<RefCell<JsRuntime>>,
}

impl IsolatedRuntime {
  pub fn new(mut options: RuntimeOptions) -> Result<IsolatedRuntime> {
    tokio::task::spawn_local(async {
      trace!("")
      // span a noop task to make sure runtime is started in a local taskset
      // this will guarantee that ops can spawn local tasks from extensions
      // and other startup code
    });
    let permissions = options.permissions.clone();
    let runtime_config = options.config.clone();
    let user_agent = options
      .user_agent
      .as_ref()
      .unwrap_or(&"arena/runtime".to_owned())
      .to_string();
    let mut extensions = vec![
      deno_webidl::deno_webidl::init_ops(),
      deno_console::deno_console::init_ops(),
      deno_url::deno_url::init_ops(),
      deno_web::deno_web::init_ops::<PermissionsContainer>(
        Arc::new(deno_web::BlobStore::default()),
        Default::default(),
      ),
      deno_fetch::deno_fetch::init_ops::<PermissionsContainer>(
        deno_fetch::Options {
          user_agent: user_agent.clone(),
          root_cert_store_provider: None,
          proxy: None,
          request_builder_hook: None,
          unsafely_ignore_certificate_errors: None,
          client_cert_chain_and_key: None,
          file_fetch_handler: Rc::new(deno_fetch::DefaultFileFetchHandler),
        },
      ),
      Extension {
        name: "arena/core/permissions",
        op_state_fn: Some(Box::new(move |state| {
          state.put::<PermissionsContainer>(permissions.to_owned());
          state.put::<RuntimeConfig>(runtime_config);
        })),
        enabled: true,
        ..Default::default()
      },
      set_fetch_client_with_egress(user_agent, &options.config),
      Self::get_setup_extension(&options),
    ];

    let builtin_modules: Vec<String> =
      BuiltinExtensions::get_specifiers(&options.builtin_extensions)
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    extensions.extend(BuiltinExtensions::get_deno_extensions(
      &mut options.builtin_extensions,
    ));

    // Note(sagar): take extensions out of the config and set it to empty
    // vec![] so that config can be stored without having Send trait
    if options.extensions.len() > 0 {
      let exts = options.extensions;
      extensions.extend(exts);
      options.extensions = vec![];
    }

    let create_params = options.heap_limits.map(|(initial, max)| {
      v8::Isolate::create_params().heap_limits(initial, max)
    });

    let mut js_runtime = JsRuntime::new(deno_core::RuntimeOptions {
      startup_snapshot: Some(Snapshot::Static(BASE_RUNTIME_SNAPSHOT)),
      create_params,
      module_loader: Some(Rc::new(DefaultModuleLoader::new(
        builtin_modules,
        options.module_loader,
      ))),
      // Note(sagar) Since the following extensions were snapshotted, pass them
      // as `extensions` instead of `extensions_with_js`; only rust bindings are
      // necessary since JS is already loaded
      extensions,
      ..Default::default()
    });

    // Note(sagar): if the heap limits are set, terminate the runtime manually
    if options.heap_limits.is_some() {
      let cb_handle = js_runtime.v8_isolate().thread_safe_handle();
      js_runtime.add_near_heap_limit_callback(
        move |current_limit, _initial_limit| {
          error!("Terminating V8 due to memory limit");
          cb_handle.terminate_execution();
          current_limit
        },
      );
    }

    BuiltinExtensions::load_modules(
      &options.builtin_extensions,
      &mut js_runtime,
    )?;

    if !options.enable_arena_global {
      cleanup_global_arena_namespace(&mut js_runtime)?;
    }

    let runtime = IsolatedRuntime {
      runtime: Rc::new(RefCell::new(js_runtime)),
    };
    Ok(runtime)
  }

  #[allow(dead_code)]
  pub async fn execute_main_module(&mut self, url: &Url) -> Result<()> {
    let mut runtime = self.runtime.borrow_mut();
    let mod_id = runtime.load_main_module(url, None).await?;
    let receiver = runtime.mod_evaluate(mod_id);

    runtime.run_event_loop(Default::default()).await?;
    receiver.await
  }

  #[allow(dead_code)]
  pub async fn load_and_evaluate_side_module(
    &mut self,
    url: &Url,
    code: String,
  ) -> Result<()> {
    let mut runtime = self.runtime.borrow_mut();
    let mod_id = runtime
      .load_side_module(url, Some(FastString::Arc(code.into())))
      .await?;
    let receiver = runtime.mod_evaluate(mod_id);

    runtime.run_event_loop(Default::default()).await?;
    receiver.await
  }

  /// Note: the caller is responsbile for running the event loop and
  /// calling await on the receiver. For example:
  /// ```
  /// runtime.run_event_loop(Default::default()).await?;
  /// receiver.await
  /// ```
  #[allow(dead_code)]
  pub async fn execute_main_module_code(
    &mut self,
    url: &Url,
    code: &str,
  ) -> Result<()> {
    let mut runtime = self.runtime.borrow_mut();
    let mod_id = runtime
      .load_main_module(url, Some(code.to_owned().into()))
      .await?;
    let receiver = runtime.mod_evaluate(mod_id);

    runtime.run_event_loop(Default::default()).await?;
    receiver.await
  }

  pub async fn run_event_loop(&mut self) -> Result<()> {
    let mut runtime = self.runtime.borrow_mut();
    runtime.run_event_loop(Default::default()).await
  }

  #[allow(dead_code)]
  pub fn execute_script(
    &mut self,
    name: &'static str,
    code: &str,
  ) -> Result<v8::Global<v8::Value>> {
    let mut runtime = self.runtime.borrow_mut();
    runtime
      .execute_script(name, FastString::Owned(code.to_owned().into()))
      .map_err(|e| anyhow!("V8 error: {:?}", e))
  }

  fn get_setup_extension(config: &RuntimeOptions) -> Extension {
    let mut js_files = Vec::new();

    js_files.push(include_in_binary!(
      "init",
      r#"
        Arena.core = Deno.core;
        Arena.core.setMacrotaskCallback(globalThis.__bootstrap.handleTimerMacrotask);
        // TODO: remove me
        // Object.assign(globalThis.Arena, {
        //   config: Arena.core.ops.op_load_arena_config()
        // });
        // Arena.core.opAsync("op_load_arena_config_async");
      "#
    ));

    js_files.push(include_in_binary!(
      "runtime/init/console",
      r#"globalThis.console = 
        new globalThis.__bootstrap.Console(Arena.core.print);"#,
      config.enable_console
    ));

    js_files.push(include_in_binary!(
      "runtime/init/finalize",
      r#"
        // Remove bootstrapping data from the global scope
        delete globalThis.__bootstrap;
        delete globalThis.bootstrap;
      "#
    ));

    Extension {
      name: "arena/runtime/init",
      js_files: js_files.into(),
      enabled: true,
      ..Default::default()
    }
  }
}

fn set_fetch_client_with_egress(
  user_agent: String,
  config: &RuntimeConfig,
) -> Extension {
  let egress_addr = config.egress_addr.clone();
  Extension {
    name: "rutime/init/fetch",
    op_state_fn: Some(Box::new(move |state| {
      if let Some(egress_addr) = egress_addr {
        let mut client = utils::fetch::get_default_http_client_builder(
          &user_agent,
          CreateHttpClientOptions {
            root_cert_store: None,
            ca_certs: vec![],
            proxy: None,
            unsafely_ignore_certificate_errors: None,
            client_cert_chain_and_key: None,
            pool_max_idle_per_host: None,
            pool_idle_timeout: None,
            http1: true,
            http2: true,
          },
        )
        .unwrap();
        client = client.local_address(egress_addr);
        state.put::<reqwest::Client>(client.build().unwrap());
      }
    })),
    ..Default::default()
  }
}

fn cleanup_global_arena_namespace(runtime: &mut JsRuntime) -> Result<()> {
  futures::executor::block_on(async {
    runtime.execute_script(
      "<setup/global/reset>",
      FastString::Static(
        r#"
        // Delete reference to global Arena that has lots of runtime features
        // and only provide access to select few features/configs
        let newArena = {
          // TODO
          // config: Arena.config,
          fs: Arena.fs,
        };
        delete globalThis["Deno"];
        delete globalThis["Arena"];
        globalThis.Arena = newArena;
        "#,
      ),
    )?;
    runtime.run_event_loop(Default::default()).await?;
    Ok::<(), anyhow::Error>(())
  })
}
