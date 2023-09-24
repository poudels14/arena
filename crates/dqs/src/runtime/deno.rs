use anyhow::Result;
use cloud::pubsub::exchange::Exchange;
use cloud::pubsub::Node;
use cloud::CloudExtensionProvider;
use common::deno::extensions::server::HttpServerConfig;
use common::deno::extensions::{BuiltinExtensions, BuiltinModule};
use common::deno::loader::BuiltInModuleLoader;
use deno_core::{
  v8, Extension, ExtensionFileSource, ExtensionFileSourceCode, JsRuntime,
  ModuleLoader, Snapshot,
};
use deno_fetch::CreateHttpClientOptions;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use jsruntime::permissions::PermissionsContainer;
use std::net::IpAddr;
use std::rc::Rc;
use tracing::error;

use crate::arena::{ArenaRuntimeState, MainModule};
use crate::loaders::moduleloader::AppkitModuleLoader;
use crate::loaders::template::TemplateLoader;

pub static WORKSPACE_DQS_SNAPSHOT: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/WORKSPACE_DQS_SNAPSHOT.bin"));

#[derive(Clone, Debug)]
pub struct RuntimeOptions {
  /// Runtime id
  pub id: String,
  pub db_pool: Option<Pool<ConnectionManager<PgConnection>>>,
  pub server_config: HttpServerConfig,
  pub exchange: Option<Exchange>,
  pub permissions: PermissionsContainer,
  /// Heap limit tuple: (initial size, max hard limit) in bytes
  pub heap_limits: Option<(usize, usize)>,
  /// The local address to use for outgoing network request
  /// This is useful if we need to restrict the outgoing network
  /// request to a specific network device/address
  pub egress_address: Option<IpAddr>,
  pub state: ArenaRuntimeState,
}

pub async fn new(config: RuntimeOptions) -> Result<JsRuntime> {
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
    self::build_extension(&config),
  ];

  let publisher = if let Some(exchange) = config.exchange {
    let node = match &config.state.module {
      MainModule::App { app } => Node::App { id: app.id.clone() },
      MainModule::Workflow {
        id,
        name: _,
        plugin: _,
      } => Node::Workflow { id: id.to_string() },
      _ => Node::Unknown,
    };
    Some(exchange.new_publisher(node).await)
  } else {
    None
  };

  let mut builtin_extensions = BuiltinExtensions::with_modules(
    vec![
      vec![
        BuiltinModule::Node(Some(vec!["crypto"])),
        BuiltinModule::Postgres,
        BuiltinModule::Sqlite,
        BuiltinModule::HttpServer(config.server_config),
        BuiltinModule::UsingProvider(Rc::new(CloudExtensionProvider {
          publisher,
        })),
      ],
      config.state.module.get_builtin_module_extensions(),
    ]
    .concat(),
  );
  extensions.extend(builtin_extensions.deno_extensions());

  let create_params = config.heap_limits.map(|(initial, max)| {
    v8::Isolate::create_params().heap_limits(initial, max)
  });

  let module_loader: Option<Rc<dyn ModuleLoader>> = match config.db_pool.clone()
  {
    Some(db_pool) => Some(Rc::new(AppkitModuleLoader {
      workspace_id: config.state.workspace_id,
      pool: db_pool,
      template_loader: TemplateLoader {
        module: config.state.module,
        registry: config.state.registry,
      },
    })),
    None => Some(Rc::new(BuiltInModuleLoader {})),
  };

  let mut runtime = JsRuntime::new(deno_core::RuntimeOptions {
    startup_snapshot: Some(Snapshot::Static(WORKSPACE_DQS_SNAPSHOT)),
    create_params,
    module_loader,
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

fn build_extension(config: &RuntimeOptions) -> Extension {
  let user_agent = get_user_agent(&config.id);
  let egress_address = config.egress_address.clone();
  let state = config.state.clone();
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
        op_state.put::<ArenaRuntimeState>(state);
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
