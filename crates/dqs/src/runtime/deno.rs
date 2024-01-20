use std::net::IpAddr;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Result;
use cloud::identity::Identity;
use cloud::pubsub::exchange::Exchange;
use cloud::CloudExtensionProvider;
use deno_core::{
  v8, Extension, ExtensionFileSource, ExtensionFileSourceCode, JsRuntime,
  Snapshot,
};
use deno_fetch::CreateHttpClientOptions;
use derivative::Derivative;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use runtime::extensions::server::HttpServerConfig;
use runtime::extensions::{
  BuiltinExtensionProvider, BuiltinExtensions, BuiltinModule,
};
use runtime::permissions::PermissionsContainer;
use tracing::error;

use crate::arena::{self, ArenaRuntimeState, MainModule};
use crate::loaders::moduleloader::AppkitModuleLoader;
use crate::loaders::template::TemplateLoader;

pub static WORKSPACE_DQS_SNAPSHOT: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/WORKSPACE_DQS_SNAPSHOT.bin"));

#[derive(Clone, Derivative)]
#[derivative(Debug)]
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

  #[derivative(Debug = "ignore")]
  pub template_loader: Arc<dyn TemplateLoader>,

  pub state: ArenaRuntimeState,
}

pub async fn new(config: RuntimeOptions) -> Result<JsRuntime> {
  let mut extensions = vec![
    deno_webidl::deno_webidl::init_ops(),
    deno_console::deno_console::init_ops(),
    deno_url::deno_url::init_ops(),
    deno_web::deno_web::init_ops::<PermissionsContainer>(
      deno_web::BlobStore::default().into(),
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
  ];

  let publisher = if let Some(exchange) = &config.exchange {
    let node = match &config.state.module {
      MainModule::App { app } => Identity::App {
        id: app.id.clone(),
        system_originated: None,
      },
      MainModule::PluginWorkflowRun { workflow } => Identity::WorkflowRun {
        id: workflow.id.to_string(),
        system_originated: None,
      },
      _ => Identity::Unknown,
    };
    Some(exchange.new_publisher(node).await)
  } else {
    None
  };

  let mut builtin_extensions = vec![
    BuiltinModule::Node(Some(vec!["crypto"])),
    BuiltinModule::Postgres,
    BuiltinModule::HttpServer(config.server_config.clone()),
    BuiltinModule::UsingProvider(Rc::new(CloudExtensionProvider { publisher })),
    BuiltinModule::Custom(Rc::new(arena::extension)),
  ]
  .iter()
  .map(|m| m.get_extension())
  .collect();

  extensions.extend(BuiltinExtensions::get_deno_extensions(
    &mut builtin_extensions,
  ));
  extensions.push(self::build_init_extension(&config));

  let create_params = config.heap_limits.map(|(initial, max)| {
    v8::Isolate::create_params().heap_limits(initial, max)
  });

  let mut runtime = JsRuntime::new(deno_core::RuntimeOptions {
    startup_snapshot: Some(Snapshot::Static(WORKSPACE_DQS_SNAPSHOT)),
    create_params,
    module_loader: Some(Rc::new(AppkitModuleLoader {
      workspace_id: config.state.workspace_id,
      pool: config.db_pool.clone(),
      template_loader: config.template_loader,
    })),
    extensions,
    ..Default::default()
  });

  BuiltinExtensions::load_extensions(&builtin_extensions, &mut runtime)?;

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

fn build_init_extension(config: &RuntimeOptions) -> Extension {
  let user_agent = get_user_agent(&config.id);
  let egress_address = config.egress_address.clone();
  let state = config.state.clone();
  let permissions = config.permissions.clone();

  Extension {
    name: "workspace/runtime",
    js_files: vec![ExtensionFileSource {
      specifier: "init",
      code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "./init.js"
      )),
    }]
    .into(),
    op_state_fn: Some(Box::new(move |op_state| {
      op_state.put::<ArenaRuntimeState>(state);
      op_state.put::<PermissionsContainer>(permissions);

      if let Some(egress_address) = egress_address {
        let mut client =
          runtime::utils::fetch::get_default_http_client_builder(
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
    })),
    enabled: true,
    ..Default::default()
  }
}

fn get_user_agent(id: &str) -> String {
  format!("arena/{}", id)
}
