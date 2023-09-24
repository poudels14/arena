use super::loaders;
use anyhow::{anyhow, bail, Result};
use common::arena::ArenaConfig;
use common::deno::extensions::BuiltinExtensions;
use common::deno::permissions::PermissionsContainer;
use common::deno::resolver::fs::FsModuleResolver;
use common::deno::{self, RuntimeConfig};
use deno_core::{
  op, v8, ExtensionFileSource, ExtensionFileSourceCode, FastString, JsRealm,
  ModuleLoader, OpState,
};
use deno_core::{Extension, JsRuntime, Snapshot};
use deno_fetch::CreateHttpClientOptions;
use derivative::Derivative;
use serde_json::{json, Value};
use std::cell::RefCell;
use std::net::IpAddr;
use std::path::PathBuf;
use std::rc::Rc;
use tracing::error;
use url::Url;

pub static RUNTIME_PROD_SNAPSHOT: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/RUNTIME_PROD_SNAPSHOT.bin"));

#[derive(Derivative)]
#[derivative(Default)]
pub struct RuntimeOptions {
  pub enable_console: bool,

  /// Name of the HTTP user_agent
  pub user_agent: Option<String>,

  /// The local address to use for outgoing network request
  /// This is useful if we need to restrict the outgoing network
  /// request to a specific network device/address
  pub egress_addr: Option<IpAddr>,

  pub permissions: PermissionsContainer,

  /// Heap limit tuple: (initial size, max hard limit) in bytes
  pub heap_limits: Option<(usize, usize)>,

  /// Additional extensions to add to the runtime
  pub extensions: Vec<Extension>,

  pub builtin_extensions: BuiltinExtensions,

  /// whether to auto-transpile the code when loading
  pub transpile: bool,

  pub disable_module_loader: bool,

  /// Project root must be passed
  /// This should either be a directory where package.json is located
  /// or current directory
  /// Use {@link has_file_in_file_tree(Some(&cwd), "package.json")}
  /// to find the directory with package.json in file hierarchy
  pub project_root: Option<PathBuf>,

  /// Arena config to be used for the runtime
  /// If None is passed, package.json is checked
  /// in the current directory as well as up the directory tree
  #[derivative(Default(value = "Option::None"))]
  pub config: Option<ArenaConfig>,

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
    if options.project_root.is_none() {
      bail!("options.project_root must be set");
    } else if options.config.is_none() {
      bail!("options.config must be set");
    }

    let permissions = options.permissions.clone();
    let config = RuntimeConfig {
      project_root: options.project_root.clone().unwrap(),
    };

    let arena_config = options.config.clone().unwrap_or_default();
    let user_agent = options
      .user_agent
      .as_ref()
      .unwrap_or(&"arena/runtime".to_owned())
      .to_string();
    let egress_addr = options.egress_addr.clone();
    let mut extensions = vec![
      deno_webidl::deno_webidl::init_ops(),
      deno_console::deno_console::init_ops(),
      deno_url::deno_url::init_ops(),
      deno_web::deno_web::init_ops::<PermissionsContainer>(
        deno_web::BlobStore::default(),
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
      Extension::builder("arena/core/permissions")
        .state(move |state| {
          state.put::<PermissionsContainer>(permissions.to_owned());
          state.put::<RuntimeConfig>(config);
        })
        .build(),
      // Note(sagar): put ArenaConfig in the state so that other extensions
      // can use it
      Extension::builder("arena/config")
        .state(move |state| {
          state.put::<ArenaConfig>(arena_config.to_owned());
          if let Some(egress_addr) = egress_addr {
            let mut client = deno::fetch::get_default_http_client_builder(
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
        })
        .build(),
      Self::get_setup_extension(&options),
    ];

    extensions.extend(options.builtin_extensions.deno_extensions());

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

    let module_loader: Option<Rc<dyn ModuleLoader>> =
      if options.disable_module_loader {
        None
      } else {
        let builtin_modules: Vec<String> = options
          .builtin_extensions
          .get_specifiers()
          .iter()
          .map(|s| s.to_string())
          .collect::<Vec<String>>();
        // Note(sagar): module loader should be disabled for deployed app
        Some(Rc::new(loaders::FsModuleLoader::new(
          loaders::ModuleLoaderOption {
            transpile: options.transpile,
            resolver: FsModuleResolver::new(
              options.project_root.clone().unwrap(),
              options
                .config
                .as_ref()
                // Note(sagar): use c.server.javascript since it is a merged
                // copy of c.javascript and c.server.javascript and `c.server`
                // is meant for server runtime anyways
                .and_then(|c| c.server.javascript.as_ref())
                .and_then(|j| j.resolve.clone())
                .unwrap_or(Default::default()),
              builtin_modules,
            ),
          },
        )))
      };

    let mut js_runtime = JsRuntime::new(deno_core::RuntimeOptions {
      // TODO(sagar): remove build snapshot from deployed app runner to save memory
      startup_snapshot: Some(Snapshot::Static(RUNTIME_PROD_SNAPSHOT)),
      create_params,
      module_loader,
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

    options
      .builtin_extensions
      .load_runtime_modules(&mut js_runtime)?;

    if !options.enable_arena_global {
      futures::executor::block_on(async {
        js_runtime.execute_script(
            "<setup/global/reset>",
            FastString::Static(
              r#"
              // Delete reference to global Arena that has lots of runtime features
              // and only provide access to select few features/configs
              let newArena = {
                config: Arena.config,
                fs: Arena.fs,
              };

              delete globalThis["Deno"];
              delete globalThis["Arena"];
              globalThis.Arena = newArena;
              "#,
            ),
          )?;
        js_runtime.run_event_loop(false).await?;
        Ok::<(), anyhow::Error>(())
      })?;
    }

    let runtime = IsolatedRuntime {
      runtime: Rc::new(RefCell::new(js_runtime)),
    };

    Ok(runtime)
  }

  pub async fn execute_main_module(&mut self, url: &Url) -> Result<()> {
    let mut runtime = self.runtime.borrow_mut();
    let mod_id = runtime.load_main_module(url, None).await?;
    let receiver = runtime.mod_evaluate(mod_id);

    runtime.run_event_loop(false).await?;
    receiver.await?
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

    runtime.run_event_loop(false).await?;
    receiver.await?
  }

  /// Note: the caller is responsbile for running the event loop and
  /// calling await on the receiver. For example:
  /// ```
  /// runtime.run_event_loop(false).await?;
  /// receiver.await?
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
    name: &'static str,
    code: &str,
  ) -> Result<v8::Global<v8::Value>> {
    let mut runtime = self.runtime.borrow_mut();
    runtime
      .execute_script(name, FastString::Owned(code.to_owned().into()))
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

  fn get_setup_extension(config: &RuntimeOptions) -> Extension {
    let mut js_files = Vec::new();

    js_files.push(ExtensionFileSource {
      specifier: "init",
      code: ExtensionFileSourceCode::IncludedInBinary(
        r#"
        Arena.core = Deno.core;
        Arena.core.setMacrotaskCallback(globalThis.__bootstrap.handleTimerMacrotask);
        Object.assign(globalThis.Arena, {
          config: Arena.core.ops.op_load_arena_config()
        });
      "#,
      ),
    });

    if config.enable_console {
      js_files.push(ExtensionFileSource {
        specifier: "console",
        code: ExtensionFileSourceCode::IncludedInBinary(r#"
          globalThis.console = new globalThis.__bootstrap.Console(Arena.core.print);
        "#),
      });
    }

    js_files.push(ExtensionFileSource {
      specifier: "init/finalize",
      code: ExtensionFileSourceCode::IncludedInBinary(
        r#"
        // Remove bootstrapping data from the global scope
        delete globalThis.__bootstrap;
        delete globalThis.bootstrap;
      "#,
      ),
    });

    Extension::builder("arena")
      .ops(vec![op_load_arena_config::decl()])
      .js(js_files)
      .force_op_registration()
      .build()
  }
}

#[op]
pub fn op_load_arena_config(state: &mut OpState) -> Result<Value> {
  let config = state.borrow_mut::<ArenaConfig>();
  Ok(json!(config))
}
