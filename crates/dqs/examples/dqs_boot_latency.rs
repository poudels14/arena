use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use anyhow::Result;
use common::deno::extensions::server::HttpServerConfig;
use dqs::arena::{ArenaRuntimeState, MainModule};
use dqs::loaders::Registry;
use dqs::runtime::deno;
use tokio::sync::mpsc;

fn main() -> Result<()> {
  let start = Instant::now();

  let rt = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()?;

  let _ = rt.block_on(async {
    let (_, http_requests_rx) = mpsc::channel(5);

    let main_module = MainModule::Inline {
      code: "1+1".to_owned(),
    };

    let mut runtime = deno::new(deno::RuntimeOptions {
      id: "test_runtime".to_string(),
      db_pool: None,
      server_config: HttpServerConfig::Stream(Rc::new(RefCell::new(
        http_requests_rx,
      ))),
      permissions: Default::default(),
      heap_limits: None,
      egress_address: None,
      exchange: None,
      state: ArenaRuntimeState {
        workspace_id: "test_workspace".to_string(),
        root: None,
        registry: Registry {
          host: "".to_string(),
          api_key: "".to_string(),
        },
        module: main_module.clone(),
        env_variables: Default::default(),
      },
    })
    .await?;

    let entry_module = main_module.get_entry_module()?;

    let mod_id = runtime
      .load_main_module(&entry_module.0, entry_module.1)
      .await?;

    let rx = runtime.mod_evaluate(mod_id);
    runtime.run_event_loop(false).await?;
    rx.await??;

    Ok::<(), anyhow::Error>(())
  })?;

  println!(
    "Time taken = {}",
    Instant::now().duration_since(start).as_millis()
  );

  Ok(())
}
