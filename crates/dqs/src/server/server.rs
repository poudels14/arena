use super::runtime::{self, RuntimeConfig};
use anyhow::{anyhow, Result};
use common::beam;
use deno_core::v8::IsolateHandle;
use deno_core::JsRuntime;
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info};
use url::Url;

#[derive(Debug)]
pub enum ServerEvents {
  Started(IsolateHandle, beam::Sender<Command, Value>),
  Terminated,
}

#[derive(Debug, Clone)]
pub enum Command {
  Ping,
  Terminate,
}

pub(crate) fn start(
  config: RuntimeConfig,
  tx: oneshot::Sender<mpsc::Receiver<ServerEvents>>,
) -> Result<()> {
  let (events_tx, events_rx) = mpsc::channel(5);
  tx.send(events_rx).unwrap();
  let rt = tokio::runtime::Builder::new_current_thread()
    .enable_io()
    .enable_time()
    .worker_threads(1)
    // TODO(sagar): optimize max blocking threads
    .max_blocking_threads(1)
    .build()?;

  let local = tokio::task::LocalSet::new();
  local.block_on(&rt, async {
    let mut runtime = runtime::new(config).await?;
    let (sender, receiver) = beam::channel(10);
    let (terminate_tx, terminate_rx) = oneshot::channel::<()>();

    events_tx
      .send(ServerEvents::Started(
        runtime.v8_isolate().thread_safe_handle(),
        sender,
      ))
      .await?;
    local
      .spawn_local(async { listen_to_commands(receiver, terminate_tx).await });

    info!("DQS server started...");
    tokio::select! {
      _terminate = terminate_rx => {
        info!("terinating DQS server...");
      },
      _runtime_res = run_dqs_server(runtime) => {}
    }
    info!("DQS server stopped");
    events_tx.send(ServerEvents::Terminated).await?;
    Ok(())
  })
}

async fn run_dqs_server(mut runtime: JsRuntime) -> Result<()> {
  let mod_id = runtime
    .load_main_module(
      &Url::parse("file:///@arena/workspace/main")?,
      Some(include_str!("./server.js").to_owned()),
    )
    .await?;

  let rx = runtime.mod_evaluate(mod_id);
  runtime.run_event_loop(false).await?;
  rx.await??;
  Ok(())
}

async fn listen_to_commands(
  mut receiver: beam::Receiver<Command, Value>,
  terminate_tx: oneshot::Sender<()>,
) -> Result<()> {
  while let Some((cmd, tx)) = receiver.recv().await {
    debug!("command received: {:?}", cmd);
    match cmd {
      Command::Ping => {
        tx.send(Value::String("PONG".to_owned())).unwrap();
      }
      Command::Terminate => {
        let res = terminate_tx
          .send(())
          .map(|_| Value::Bool(true))
          .unwrap_or(Value::Bool(false));
        return tx.send(res).map_err(|e| anyhow!("{:?}", e));
      }
    }
  }

  Ok(())
}
