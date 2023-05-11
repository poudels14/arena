use crate::events::ServerEvent;
use crate::runtime::{self, RuntimeConfig};
use anyhow::{anyhow, Result};
use common::beam::{self, Handle};
use deno_core::v8::IsolateHandle;
use deno_core::Resource;
use derivative::Derivative;
use serde_json::Value;
use std::rc::Rc;
use std::thread::{self, JoinHandle};
use tokio::sync::oneshot;
use url::Url;

#[derive(Derivative)]
pub struct WorkspaceServerHandle {
  pub config: RuntimeConfig,
  pub events: beam::Handle<Value, ServerEvent, Value>,
  pub thread_handle: JoinHandle<()>,
  pub isolate_handle: IsolateHandle,
}

pub(crate) async fn new(
  config: runtime::RuntimeConfig,
) -> Result<WorkspaceServerHandle> {
  let (client, server) = beam::channel::new::<Value, ServerEvent, Value>(10);
  let (tx, rx) = oneshot::channel();
  let config_clone = config.clone();
  let thread_handle = thread::spawn(move || {
    let rt = tokio::runtime::Builder::new_current_thread()
      .enable_io()
      .enable_time()
      .worker_threads(1)
      // TODO(sagar): optimize max blocking threads
      .max_blocking_threads(2)
      .build()
      .unwrap();

    let local = tokio::task::LocalSet::new();
    local.block_on(&rt, async {
      match start_workspace_server(config, server, tx).await {
        Err(e) => {
          println!("Error: {:?}", e);
        }
        _ => {
          println!("sleeping...");
          std::thread::sleep(std::time::Duration::from_secs(2));
          println!("woke up");
        }
      }
    })
  });

  let isolate_handle = rx.await?;

  Ok(WorkspaceServerHandle {
    config: config_clone,
    isolate_handle,
    thread_handle,
    events: client,
  })
}

async fn start_workspace_server(
  config: runtime::RuntimeConfig,
  events: Handle<ServerEvent, Value, Value>,
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

impl Resource for WorkspaceServerHandle {
  fn close(self: Rc<Self>) {
    drop(self);
  }
}
