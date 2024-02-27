use std::net::IpAddr;
use std::rc::Rc;
use std::str::FromStr;

use anyhow::Result;
use deno_core::{
  v8, Extension, ExtensionFileSource, ExtensionFileSourceCode, JsRuntime,
  ModuleLoader, Snapshot,
};
use deno_fetch::CreateHttpClientOptions;
use derivative::Derivative;
use http::{HeaderMap, HeaderName, HeaderValue};
use runtime::extensions::{
  BuiltinExtensionProvider, BuiltinExtensions, BuiltinModule,
};
use runtime::permissions::PermissionsContainer;
use runtime::DefaultModuleLoader;
use tracing::error;

pub static DQS_SNAPSHOT: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/WORKSPACE_DQS_SNAPSHOT.bin"));

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct RuntimeOptions<State> {
  /// Runtime id
  pub id: String,
  pub v8_platform: v8::SharedRef<v8::Platform>,
  // pub server_config: HttpServerConfig,
  pub permissions: PermissionsContainer,
  /// Heap limit tuple: (initial size, max hard limit) in bytes
  pub heap_limits: Option<(usize, usize)>,
  /// The local address to use for outgoing network request
  /// This is useful if we need to restrict the outgoing network
  /// request to a specific network device/address
  pub egress_address: Option<IpAddr>,

  /// Default egress headers
  pub egress_headers: Option<Vec<(String, String)>>,

  #[derivative(Debug = "ignore")]
  pub module_loader: Option<Rc<dyn ModuleLoader>>,

  #[derivative(Debug = "ignore")]
  pub modules: Vec<BuiltinModule>,
  pub state: Option<State>,
}

pub async fn create_new<State>(
  config: RuntimeOptions<State>,
) -> Result<JsRuntime>
where
  State: Clone + 'static,
{
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

  let mut builtin_extensions = vec![
    BuiltinModule::Node(Some(vec!["crypto"])),
    BuiltinModule::Postgres,
    BuiltinModule::Cloudflare,
  ]
  .iter()
  .map(|m| m.get_extension())
  .collect();

  extensions.extend(BuiltinExtensions::get_deno_extensions(
    &mut builtin_extensions,
  ));
  extensions.extend(BuiltinExtensions::get_deno_extensions(
    &mut config.modules.iter().map(|m| m.get_extension()).collect(),
  ));
  extensions.push(self::build_init_extension(&config));

  let create_params = config.heap_limits.map(|(initial, max)| {
    v8::Isolate::create_params().heap_limits(initial, max)
  });

  let builtin_modules: Vec<String> =
    BuiltinExtensions::get_specifiers(&builtin_extensions)
      .iter()
      .map(|s| s.to_string())
      .collect::<Vec<String>>();

  let mut runtime = JsRuntime::new(deno_core::RuntimeOptions {
    startup_snapshot: Some(Snapshot::Static(DQS_SNAPSHOT)),
    v8_platform: Some(config.v8_platform),
    create_params,
    module_loader: Some(config.module_loader.unwrap_or_else(|| {
      Rc::new(DefaultModuleLoader::new(builtin_modules, None))
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

fn build_init_extension<State>(config: &RuntimeOptions<State>) -> Extension
where
  State: Clone + 'static,
{
  let user_agent = get_user_agent(&config.id);
  let egress_address = config.egress_address.clone();
  let egress_headers = config.egress_headers.clone().unwrap_or_default();
  let state = config.state.clone();
  let permissions = config.permissions.clone();

  Extension {
    name: "dqs/runtime",
    js_files: vec![ExtensionFileSource {
      specifier: "init",
      code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "./init.js"
      )),
    }]
    .into(),
    op_state_fn: Some(Box::new(move |op_state| {
      if let Some(state) = state {
        op_state.put::<State>(state);
      }
      op_state.put::<PermissionsContainer>(permissions);
      let mut client = runtime::utils::fetch::get_default_http_client_builder(
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

      let mut default_headers = HeaderMap::new();
      egress_headers.into_iter().for_each(|(key, value)| {
        if let (Ok(key), Ok(value)) =
          (HeaderName::from_str(&key), HeaderValue::from_str(&value))
        {
          default_headers.insert(key, value);
        }
      });
      client = client.default_headers(default_headers);
      if let Some(egress_addr) = egress_address {
        client = client.local_address(egress_addr);
      }
      client = client.local_address(egress_address);
      op_state.put::<reqwest::Client>(client.build().unwrap());
    })),
    enabled: true,
    ..Default::default()
  }
}

fn get_user_agent(id: &str) -> String {
  format!("arena/{}", id)
}
