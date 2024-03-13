use std::net::IpAddr;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Result;
use cloud::identity::Identity;
use cloud::pubsub::exchange::Exchange;
use cloud::rowacl::RowAclChecker;
use cloud::CloudExtensionProvider;
use deno_core::{v8, JsRuntime};
use derivative::Derivative;
use parking_lot::RwLock;
use runtime::extensions::server::HttpServerConfig;
use runtime::extensions::BuiltinModule;
use runtime::permissions::PermissionsContainer;
use sqlx::{Pool, Postgres};

use super::core;
use crate::arena::{self, ArenaRuntimeState, MainModule};
use crate::loaders::moduleloader::AppkitModuleLoader;
use crate::loaders::template::TemplateLoader;

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct RuntimeOptions {
  /// Runtime id
  pub id: String,
  pub db_pool: Option<Pool<Postgres>>,
  pub v8_platform: v8::SharedRef<v8::Platform>,
  pub server_config: Option<HttpServerConfig>,
  pub exchange: Option<Exchange>,
  pub acl_checker: Option<Arc<RwLock<RowAclChecker>>>,
  pub permissions: PermissionsContainer,
  /// Heap limit tuple: (initial size, max hard limit) in bytes
  pub heap_limits: Option<(usize, usize)>,
  /// The local address to use for outgoing network request
  /// This is useful if we need to restrict the outgoing network
  /// request to a specific network device/address
  pub egress_address: Option<IpAddr>,
  /// Default egress headers
  pub egress_headers: Option<Vec<(String, String)>>,

  pub module: MainModule,

  #[derivative(Debug = "ignore")]
  pub template_loader: Arc<dyn TemplateLoader>,

  pub state: ArenaRuntimeState,
  /// Identity of the app server
  pub identity: Identity,
}

pub async fn new(config: RuntimeOptions) -> Result<JsRuntime> {
  let publisher = if let Some(exchange) = &config.exchange {
    Some(exchange.new_publisher(config.identity).await)
  } else {
    None
  };

  let mut modules = vec![
    BuiltinModule::UsingProvider(Rc::new(CloudExtensionProvider {
      publisher,
      acl_checker: config.acl_checker,
    })),
    BuiltinModule::Custom(Rc::new(arena::extension)),
  ];
  if let Some(server_config) = config.server_config {
    modules.push(BuiltinModule::HttpServer(server_config));
  }
  core::create_new(core::RuntimeOptions {
    id: config.id,
    modules,
    egress_address: config.egress_address,
    egress_headers: config.egress_headers,
    heap_limits: config.heap_limits,
    module_loader: Some(Rc::new(AppkitModuleLoader {
      workspace_id: config.state.workspace_id.clone(),
      module: config.module.clone(),
      pool: config.db_pool.clone(),
      template_loader: config.template_loader,
    })),
    permissions: config.permissions,
    state: Some(config.state),
    v8_platform: config.v8_platform,
  })
  .await
}
