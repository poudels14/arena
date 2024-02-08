use std::net::IpAddr;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

use anyhow::Result;
use cloud::CloudExtensionProvider;
use deno_core::{v8, FastString, PollEventLoopOptions};
use once_cell::sync::Lazy;
use runtime::extensions::server::HttpServerConfig;
use runtime::extensions::BuiltinModule;
use runtime::permissions::PermissionsContainer;
use tokio::sync::oneshot;
use url::Url;
use uuid::Uuid;

use super::runtime::core;

pub struct RuntimeOptions {
  pub id: String,
  pub server_config: HttpServerConfig,
  pub heap_limits: Option<(usize, usize)>,
  pub egress_address: Option<IpAddr>,
  pub permissions: PermissionsContainer,
  pub v8_platform: v8::SharedRef<v8::Platform>,
}

static RUNTIME_COUNTER: Lazy<Arc<AtomicUsize>> =
  Lazy::new(|| Arc::new(AtomicUsize::new(1)));

pub struct PyodideRuntime {}

impl PyodideRuntime {
  pub async fn start_pyodide_runtime(options: RuntimeOptions) -> Result<()> {
    let (tx, rx) = oneshot::channel::<bool>();
    let thread = thread::Builder::new().name(format!(
      "pyodide-[{}]-{}",
      options.id,
      RUNTIME_COUNTER.fetch_add(1, Ordering::AcqRel)
    ));

    let thread_handle = thread.spawn(move || {
      let rt = tokio::runtime::Builder::new_current_thread()
        .thread_name(options.id.clone())
        .enable_io()
        .enable_time()
        .worker_threads(1)
        .build()?;

      // TODO(sagar): security
      // Do few things in prod to make sure file access is properly restricted:
      // - use chroot to limit the files this process has access to
      // - change process user and make make sure only that user has access to
      //   given files and directories
      let local = tokio::task::LocalSet::new();
      let _ = local.block_on(&rt, async {
        let mut runtime = core::create_new::<usize>(core::RuntimeOptions {
          id: Uuid::new_v4().to_string(),
          modules: vec![
            BuiltinModule::HttpServer(options.server_config),
            BuiltinModule::UsingProvider(Rc::new(CloudExtensionProvider {
              publisher: None,
            })),
          ],
          egress_address: options.egress_address,
          heap_limits: options.heap_limits,
          v8_platform: options.v8_platform,
          permissions: options.permissions,
          module_loader: None,
          state: None,
        })
        .await
        .expect("starting python runtime");

        tracing::trace!("loading main module");
        let mod_id = runtime
          .load_main_module(
            &Url::parse("file:///main").unwrap(),
            Some(FastString::Static(include_str!(
              "../../../../js/runtime/dist/cloud/pyodide/server.js"
            ))),
          )
          .await
          .expect("running python runtime setup");

        tracing::trace!("main module loaded!");
        let rx = runtime.mod_evaluate(mod_id);
        runtime
          .run_event_loop(Default::default())
          .await
          .expect("running event loop");
        rx.await.expect("running main module");

        tx.send(true)
          .expect("Error sending python runtime ready notif");
        runtime
          .run_event_loop(PollEventLoopOptions::default())
          .await
          .expect("Error running python runtime");
      });

      Ok::<(), anyhow::Error>(())
    });

    let _ = rx.await.expect("Failed to wait for python runtime");
    Ok(())
  }
}
