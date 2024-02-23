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
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use parking_lot::RwLock;
use runtime::extensions::server::HttpServerConfig;
use runtime::extensions::BuiltinModule;
use runtime::permissions::PermissionsContainer;

use super::core;
use crate::arena::{self, ArenaRuntimeState, MainModule};
use crate::loaders::moduleloader::AppkitModuleLoader;
use crate::loaders::template::TemplateLoader;

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct RuntimeOptions {
  /// Runtime id
  pub id: String,
  pub db_pool: Option<Pool<ConnectionManager<PgConnection>>>,
  pub v8_platform: v8::SharedRef<v8::Platform>,
  pub server_config: HttpServerConfig,
  pub exchange: Option<Exchange>,
  pub acl_checker: Option<Arc<RwLock<RowAclChecker>>>,
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

  core::create_new(core::RuntimeOptions {
    id: config.id,
    modules: vec![
      BuiltinModule::HttpServer(config.server_config),
      BuiltinModule::UsingProvider(Rc::new(CloudExtensionProvider {
        publisher,
        acl_checker: config.acl_checker,
      })),
      BuiltinModule::Custom(Rc::new(arena::extension)),
    ],
    egress_address: config.egress_address,
    heap_limits: config.heap_limits,
    module_loader: Some(Rc::new(AppkitModuleLoader {
      workspace_id: config.state.workspace_id.clone(),
      pool: config.db_pool.clone(),
      template_loader: config.template_loader,
    })),
    permissions: config.permissions,
    state: Some(config.state),
    v8_platform: config.v8_platform,
  })
  .await
}
