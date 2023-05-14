use crate::runtime::{self, RuntimeConfig};
use anyhow::{anyhow, Result};
use deno_core::v8::IsolateHandle;
use deno_core::Resource;
use derivative::Derivative;
use std::rc::Rc;
use std::thread::JoinHandle;
use tokio::sync::oneshot;
use url::Url;

#[derive(Derivative)]
pub struct DqsServerHandle {
  pub thread_handle: JoinHandle<Result<()>>,
  pub isolate_handle: IsolateHandle,
}

pub(crate) fn start(
  config: RuntimeConfig,
  handler_sender: oneshot::Sender<IsolateHandle>,
) -> Result<()> {
  let rt = tokio::runtime::Builder::new_current_thread()
    .enable_io()
    .enable_time()
    .worker_threads(1)
    // TODO(sagar): optimize max blocking threads
    .max_blocking_threads(1)
    .build()?;

  let local = tokio::task::LocalSet::new();
  local.block_on(&rt, async {
    match run_dqs_server(config, handler_sender).await {
      Err(e) => {
        println!("Error: {:?}", e);
      }
      _ => {
        println!("sleeping...");
        std::thread::sleep(std::time::Duration::from_secs(2));
        println!("woke up");
      }
    }
    Ok(())
  })
}

async fn run_dqs_server(
  config: RuntimeConfig,
  handler_sender: oneshot::Sender<IsolateHandle>,
) -> Result<()> {
  let mut runtime = runtime::new(config)?;
  let mod_id = runtime
    .load_main_module(
      &Url::parse("file:///@arena/workspace/main")?,
      Some(
        r#"
          import { serve } from "@arena/runtime/server";
          import { router } from "builtin:///@arena/dqs/router";

          const r = router();
          serve({
            async fetch(req) {
              return r.route(req);
            }
          });
        "#
        .to_owned(),
      ),
    )
    .await?;

  handler_sender
    .send(runtime.v8_isolate().thread_safe_handle())
    .map_err(|e| anyhow!("{:?}", e))?;

  let receiver = runtime.mod_evaluate(mod_id);
  runtime.run_event_loop(false).await?;
  receiver.await??;
  Ok(())
}

impl Resource for DqsServerHandle {
  fn close(self: Rc<Self>) {
    drop(self);
  }
}
