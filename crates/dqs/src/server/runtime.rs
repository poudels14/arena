use super::moduleloader::AppkitModuleLoader;
use super::state::RuntimeState;
use crate::apps::App;
use crate::loaders::registry::Registry;
use anyhow::Result;
use common::deno::extensions::server::HttpServerConfig;
use common::deno::extensions::{BuiltinExtensions, BuiltinModule};
use common::deno::resources::env_variable::EnvironmentVariableStore;
use deno_core::{
  v8, Extension, ExtensionFileSource, ExtensionFileSourceCode, JsRuntime,
  Snapshot,
};
use deno_fetch::CreateHttpClientOptions;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use jsruntime::permissions::PermissionsContainer;
use std::net::IpAddr;
use std::rc::Rc;
use tracing::error;

pub static WORKSPACE_DQS_SNAPSHOT: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/WORKSPACE_DQS_SNAPSHOT.bin"));

#[derive(Default, Clone, Debug)]
pub struct RuntimeOptions {
  /// Runtime id
  pub id: String,
  /// id of the workspace
  pub workspace_id: String,
  pub db_pool: Option<Pool<ConnectionManager<PgConnection>>>,
  pub server_config: HttpServerConfig,
  /// Name of the HTTP user_agent
  pub user_agent: Option<String>,
  pub permissions: PermissionsContainer,
  /// Heap limit tuple: (initial size, max hard limit) in bytes
  pub heap_limits: Option<(usize, usize)>,
  /// The local address to use for outgoing network request
  /// This is useful if we need to restrict the outgoing network
  /// request to a specific network device/address
  pub egress_address: Option<IpAddr>,
  /// Builtin modules to be loaded to the runtime
  pub modules: Vec<BuiltinModule>,
  /// App info - only set if this runtime for an app
  pub app: Option<App>,
  pub registry: Option<Registry>,
}

pub async fn new(config: RuntimeOptions) -> Result<JsRuntime> {
  let db_pool = config.db_pool.clone().unwrap();
  // TODO(sagar): instead of loading RuntimeState here, pass in as options
  let state =
    RuntimeState::init(config.workspace_id.clone(), db_pool.clone()).await?;

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
        user_agent: get_user_agent(&config.id),
        root_cert_store_provider: None,
        proxy: None,
        request_builder_hook: None,
        unsafely_ignore_certificate_errors: None,
        client_cert_chain_and_key: None,
        file_fetch_handler: Rc::new(deno_fetch::DefaultFileFetchHandler),
      },
    ),
    self::build_extension(state.clone(), &config),
  ];

  let mut builtin_extensions = BuiltinExtensions::with_modules(
    vec![
      vec![
        BuiltinModule::Node(Some(vec!["crypto"])),
        BuiltinModule::Postgres,
        BuiltinModule::Sqlite,
        BuiltinModule::HttpServer(config.server_config),
      ],
      config.modules,
    ]
    .concat(),
  );
  extensions.extend(builtin_extensions.deno_extensions());

  let create_params = config.heap_limits.map(|(initial, max)| {
    v8::Isolate::create_params().heap_limits(initial, max)
  });

  let mut runtime = JsRuntime::new(deno_core::RuntimeOptions {
    startup_snapshot: Some(Snapshot::Static(WORKSPACE_DQS_SNAPSHOT)),
    create_params,
    module_loader: Some(Rc::new(AppkitModuleLoader {
      workspace_id: config.workspace_id.clone(),
      pool: db_pool,
      state,
      app: config.app,
      registry: config.registry.expect("Registry not set"),
    })),
    extensions,
    ..Default::default()
  });

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

fn build_extension(state: RuntimeState, config: &RuntimeOptions) -> Extension {
  let user_agent = get_user_agent(&config.id);
  let egress_address = config.egress_address.clone();
  let permissions = config.permissions.clone();

  Extension::builder("workspace/runtime")
      .js(vec![
        ExtensionFileSource {
          specifier: "init",
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
        }
      ])
      .state(move |op_state| {
        op_state.put::<EnvironmentVariableStore>(state.env_variables.clone());
        op_state.put::<RuntimeState>(state);
        op_state.put::<PermissionsContainer>(permissions);

        if let Some(egress_address) = egress_address {
          let mut client = common::deno::fetch::get_default_http_client_builder(
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
          client = client.local_address(egress_address);
          op_state.put::<reqwest::Client>(client.build().unwrap());
        }
      })
      .build()
}

fn get_user_agent(id: &str) -> String {
  format!("arena/{}", id)
}
