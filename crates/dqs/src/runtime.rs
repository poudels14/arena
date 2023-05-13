use crate::loaders::{self, db};
use anyhow::Result;
use common::deno::extensions::server::HttpServerConfig;
use common::deno::extensions::{BuiltinExtensions, BuiltinModule};
use deno_core::{v8, ExtensionFileSource, ExtensionFileSourceCode, Snapshot};
use deno_core::{Extension, JsRuntime};
use derivative::Derivative;
use jsruntime::permissions::PermissionsContainer;
use std::rc::Rc;
use tracing::error;

pub static WORKSPACE_DQS_SNAPSHOT: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/WORKSPACE_DQS_SNAPSHOT.bin"));

#[derive(Derivative)]
#[derivative(Default, Clone)]
pub struct RuntimeConfig {
  pub workspace_id: String,

  pub server_config: HttpServerConfig,

  /// Name of the HTTP user_agent
  pub user_agent: Option<String>,

  pub permissions: PermissionsContainer,

  /// Heap limit tuple: (initial size, max hard limit) in bytes
  pub heap_limits: Option<(usize, usize)>,
}

pub fn new(config: RuntimeConfig) -> Result<JsRuntime> {
  let create_params = config.heap_limits.map(|(initial, max)| {
    v8::Isolate::create_params().heap_limits(initial, max)
  });

  let mut extensions = vec![
    deno_webidl::init(),
    deno_console::init(),
    deno_url::init_ops(),
    deno_web::init_ops::<PermissionsContainer>(
      deno_web::BlobStore::default(),
      Default::default(),
    ),
    deno_fetch::init_ops::<PermissionsContainer>(
      deno_fetch::Options {
        user_agent: format!("arena/dqs/{}", &config.workspace_id).to_owned(),
        root_cert_store: None,
        proxy: None,
        request_builder_hook: None,
        unsafely_ignore_certificate_errors: None,
        client_cert_chain_and_key: None,
        file_fetch_handler: Rc::new(deno_fetch::DefaultFileFetchHandler),
      },
    ),
    Extension::builder("workspace/runtime")
    .js(vec![
      ExtensionFileSource {
        specifier: "init".to_string(),
        code: ExtensionFileSourceCode::IncludedInBinary(
          r#"
          Deno.core.setMacrotaskCallback(globalThis.__bootstrap.handleTimerMacrotask);

          // TODO(sagar): remove this
          globalThis.console = new globalThis.__bootstrap.Console(Deno.core.print);

          // Remove bootstrapping data from the global scope
          delete globalThis.__bootstrap;
          delete globalThis.bootstrap;
        "#,
        ),
      }]).build()
  ];

  let mut builtin_extensions =
    BuiltinExtensions::with_modules(vec![BuiltinModule::HttpServer(
      config.server_config,
    )]);
  extensions.extend(builtin_extensions.deno_extensions());

  let mut runtime = JsRuntime::new(deno_core::RuntimeOptions {
    startup_snapshot: Some(Snapshot::Static(WORKSPACE_DQS_SNAPSHOT)),
    create_params,
    module_loader: Some(Rc::new(loaders::AppkitModuleLoader {
      workspace_id: config.workspace_id.clone(),
      pool: db::create_connection_pool(),
    })),
    extensions,
    ..Default::default()
  });

  builtin_extensions.load_snapshot_modules(&mut runtime)?;
  builtin_extensions.load_runtime_modules(&mut runtime)?;

  // Note(sagar): if the heap limits are set, terminate the runtime manually
  if config.heap_limits.is_some() {
    let cb_handle = runtime.v8_isolate().thread_safe_handle();
    runtime.add_near_heap_limit_callback(
      move |current_limit, _initial_limit| {
        error!("Terminating V8 due to memory limit");
        cb_handle.terminate_execution();
        current_limit
      },
    );
  }

  Ok(runtime)
}
