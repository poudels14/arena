use super::ext::{self, node};
use super::loaders;
use crate::buildtools;
use crate::config::ArenaConfig;
use crate::permissions::PermissionsContainer;
use anyhow::{anyhow, Result};
use common::fs::has_file_in_file_tree;
use deno_core::{
  v8, ExtensionFileSource, ExtensionFileSourceCode, JsRealm, ModuleLoader,
};
use deno_core::{Extension, JsRuntime, Snapshot};
use derivative::Derivative;
use itertools::Itertools;
use std::cell::RefCell;
use std::env::current_dir;
use std::path::PathBuf;
use std::rc::Rc;
use tracing::{debug, error};
use url::Url;

pub static RUNTIME_PROD_SNAPSHOT: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/RUNTIME_PROD_SNAPSHOT.bin"));

#[derive(Derivative)]
#[derivative(Default)]
pub struct RuntimeConfig {
  /// Arena config to be used for the runtime
  /// If None is passed, arena.config.toml is checked
  /// in the current directory as well as up the directory tree
  #[derivative(Default(value = "Option::None"))]
  pub config: Option<ArenaConfig>,

  /// Name of the HTTP user_agent
  pub user_agent: String,

  pub permissions: PermissionsContainer,

  pub enable_console: bool,

  #[derivative(Default(value = "true"))]
  pub enable_wasm: bool,

  /// Additional extensions to add to the runtime
  pub extensions: Vec<Extension>,

  /// Heap limit tuple: (initial size, max hard limit) in bytes
  pub heap_limits: Option<(usize, usize)>,

  /// enable build tools like babel, babel plugins, etc
  pub enable_build_tools: bool,

  /// whether to auto-transpile the code when loading
  pub transpile: bool,

  pub disable_module_loader: bool,

  /// enables importing from node modules like "node:fs"
  pub enable_node_modules: bool,

  pub side_modules: Vec<ExtensionFileSource>,
}

pub struct IsolatedRuntime {
  pub config: RuntimeConfig,
  pub runtime: Rc<RefCell<JsRuntime>>,
}

impl IsolatedRuntime {
  pub fn new(mut config: RuntimeConfig) -> Result<IsolatedRuntime> {
    let cwd = current_dir()?;
    let maybe_arena_config_dir =
      has_file_in_file_tree(Some(&cwd), "arena.config.toml");

    // If arena.config.toml is found, use it as project_root, else
    // use current dir
    let project_root = maybe_arena_config_dir.clone().unwrap_or(cwd);

    // If Arena config isn't passed, load from config file
    config.config = config.config.or_else(|| {
      maybe_arena_config_dir.and_then(|dir| {
        // Note(sagar): this changes Err => None, which means all errors
        // are silently ignored
        ArenaConfig::from_path(&dir.join("arena.config.toml")).ok()
      })
    });

    let permissions = config.permissions.clone();
    let mut builtin_modules = vec![];

    let mut extensions = vec![
      deno_webidl::init(),
      deno_console::init(),
      deno_url::init_ops(),
      deno_web::init_ops::<PermissionsContainer>(
        deno_web::BlobStore::default(),
        Default::default(),
      ),
      deno_crypto::init_ops(None),
      deno_fetch::init_ops::<PermissionsContainer>(deno_fetch::Options {
        user_agent: "arena/server".to_string(),
        root_cert_store: None,
        proxy: None,
        request_builder_hook: None,
        unsafely_ignore_certificate_errors: None,
        client_cert_chain_and_key: None,
        file_fetch_handler: Rc::new(deno_fetch::DefaultFileFetchHandler),
      }),
      Extension::builder("arena/core/permissions")
        .state(move |state| {
          state.put::<PermissionsContainer>(permissions.to_owned());
        })
        .build(),
    ];
    extensions.extend(Self::get_js_extensions(&project_root, &mut config));

    // Note(sagar): right now, build tools, specifically rollup requires
    // built-in node modules. so, enable then when build tools are enabled
    if config.enable_node_modules || config.enable_build_tools {
      extensions.push(ext::node::init());
      builtin_modules.extend(node::get_builtin_modules());
    }

    // Note(sagar): take extensions out of the config and set it to empty
    // vec![] so that config can be stored without having Send trait
    if config.extensions.len() > 0 {
      let exts = config.extensions;
      extensions.extend(exts);
      config.extensions = vec![];
    }

    if config.enable_build_tools {
      builtin_modules.extend(buildtools::get_build_tools_modules());
    }

    // Note(sagar): add passed in side-modules at the end so that
    // builtin modules are already loaded in case external modules depend on
    // builtin modules
    builtin_modules.extend(config.side_modules.clone());

    let create_params = config.heap_limits.map(|(initial, max)| {
      v8::Isolate::create_params().heap_limits(initial, max)
    });

    let module_loader: Option<Rc<dyn ModuleLoader>> =
      if config.disable_module_loader {
        None
      } else {
        // Note(sagar): module loader should be disabled for deployed app
        Some(Rc::new(loaders::FsModuleLoader::new(
          loaders::ModuleLoaderOption {
            transpile: config.transpile,
            resolver: super::FsModuleResolver::new(
              project_root,
              config
                .config
                .as_ref()
                .and_then(|c| c.javascript.as_ref())
                .and_then(|j| j.resolve.clone())
                .unwrap_or(Default::default()),
              builtin_modules
                .iter()
                .map(|sm| sm.specifier.clone())
                .collect(),
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

    for module in builtin_modules.iter().unique_by(|m| m.specifier.clone()) {
      futures::executor::block_on(async {
        debug!(
          "Loading built-in module into the runtime: {}",
          module.specifier
        );
        let mod_id = js_runtime
          .load_side_module(
            &Url::parse(&format!("builtin:///{}", module.specifier))?,
            Some(module.code.load()?),
          )
          .await?;
        let receiver = js_runtime.mod_evaluate(mod_id);
        js_runtime.run_event_loop(false).await?;
        receiver.await?
      })?;
    }

    let runtime = IsolatedRuntime {
      config,
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
    code: Option<String>,
  ) -> Result<()> {
    let mut runtime = self.runtime.borrow_mut();
    let mod_id = runtime.load_side_module(url, code).await?;
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

  fn get_js_extensions(
    project_root: &PathBuf,
    config: &RuntimeConfig,
  ) -> Vec<Extension> {
    let mut js_files = Vec::new();

    js_files.push(ExtensionFileSource {
      specifier: "init".to_string(),
      code: ExtensionFileSourceCode::IncludedInBinary(
        r#"
        Arena.core = Deno.core;
        Arena.core.setMacrotaskCallback(globalThis.__bootstrap.handleTimerMacrotask);
      "#,
      ),
    });

    if config.enable_console {
      js_files.push(ExtensionFileSource {
        specifier: "console".to_string(),
        code: ExtensionFileSourceCode::IncludedInBinary(r#"
          globalThis.console = new globalThis.__bootstrap.Console(Arena.core.print);
        "#),
      });
    }

    js_files.push(ExtensionFileSource {
      specifier: "init/finalize".to_string(),
      code: ExtensionFileSourceCode::IncludedInBinary(
        r#"
        // Remove bootstrapping data from the global scope
        delete globalThis.__bootstrap;
        delete globalThis.bootstrap;
      "#,
      ),
    });

    let mut extensions = vec![
      Extension::builder("arena").js(js_files).build(),
      super::ext::fs::init(),
      super::ext::env::init(config.config.as_ref().and_then(|c| c.env.clone())),
    ];
    if config.enable_wasm {
      extensions.push(super::ext::wasi::init());
    }
    if config.enable_build_tools {
      extensions.append(&mut crate::buildtools::exts::init(
        project_root,
        config
          .config
          .as_ref()
          .and_then(|c| c.javascript.as_ref())
          .and_then(|j| j.resolve.clone())
          .unwrap_or(Default::default()),
      ));
    }
    extensions
  }
}
