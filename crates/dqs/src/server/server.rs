use super::runtime::{self, RuntimeOptions};
use anyhow::{anyhow, Result};
use common::beam;
use deno_core::v8::IsolateHandle;
use deno_core::{JsRuntime, ModuleCode, ModuleSpecifier};
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};
use tracing::debug;

#[derive(Debug)]
pub enum ServerEvents {
  Started(IsolateHandle, beam::Sender<Command, Value>),
  Terminated(Result<String>),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Command {
  Ping,
  Terminate,
}

pub(crate) fn start(
  config: RuntimeOptions,
  tx: oneshot::Sender<mpsc::Receiver<ServerEvents>>,
  // server entry module `(specifier, code)`
  entry_module: (ModuleSpecifier, ModuleCode),
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

  // TODO(sagar): security
  // Do few things in prod to make sure file access is properly restricted:
  // - use chroot to limit the files this process has access to
  // - change process user and make make sure only that user has access to
  //   given files and directories

  let local = tokio::task::LocalSet::new();
  let r = local.block_on(&rt, async {
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

    debug!("DQS server started...");
    let res = tokio::select! {
      res = terminate_rx => {
        res.map(|_| "Terminated by a termination command".to_owned()).map_err(|e| anyhow!("{}", e))

      },
      res = run_dqs_server(runtime, entry_module) => {
        res.map(|_| "Terminated due to event-loop completion".to_owned()).map_err(|e| anyhow!("{}", e))
      }
    };
    debug!("DQS server stopped");
    events_tx
      .send(ServerEvents::Terminated(res))
      .await?;
    Ok(())
  });

  match r {
    Ok(()) => Ok(()),
    Err(e) => {
      // Note(sp): if the runtime was terminated because of error,
      // need to send server terminated signal
      local.block_on(&rt, async {
        events_tx
          .send(ServerEvents::Terminated(Err(anyhow!(
            "Error running DQS server: {}",
            e
          ))))
          .await
          .unwrap();
      });
      Err(e)
    }
  }
}

async fn run_dqs_server(
  mut runtime: JsRuntime,
  entry_module: (ModuleSpecifier, ModuleCode),
) -> Result<()> {
  let mod_id = runtime
    .load_main_module(&entry_module.0, Some(entry_module.1))
    .await?;

  let rx = runtime.mod_evaluate(mod_id);
  runtime.run_event_loop(false).await?;
  rx.await?
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
        debug!("DQS runtime received terminate command");
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
