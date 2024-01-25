use std::sync::Arc;

use anyhow::{anyhow, Result};
use common::beam;
use deno_core::{JsRuntime, ModuleCode, ModuleSpecifier};
use serde_json::Value;
use tokio::sync::{oneshot, watch};
use tracing::{debug, info};

use super::deno::{self, RuntimeOptions};

#[derive(Debug, Clone)]
pub enum ServerEvents {
  Init,
  Started(beam::Sender<Command, Value>),
  Terminated(Arc<Result<String>>),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Command {
  Ping,
  Terminate,
}

#[tracing::instrument(skip_all, level = "trace")]
pub(crate) fn start(
  config: RuntimeOptions,
  events_tx: watch::Sender<ServerEvents>,
) -> Result<()> {
  let rt = tokio::runtime::Builder::new_current_thread()
    .thread_name(config.id.clone())
    .enable_io()
    .enable_time()
    .worker_threads(1)
    // TODO(sagar): optimize max blocking threads
    // .max_blocking_threads(1)
    .build()?;

  // TODO(sagar): security
  // Do few things in prod to make sure file access is properly restricted:
  // - use chroot to limit the files this process has access to
  // - change process user and make make sure only that user has access to
  //   given files and directories

  let local = tokio::task::LocalSet::new();
  let r = local.block_on(&rt, async {
    let runtime = deno::new(config.clone()).await?;
    let (sender, receiver) = beam::channel(10);
    let (terminate_tx, terminate_rx) = oneshot::channel::<()>();

    events_tx
      .send(ServerEvents::Started(
        sender,
      ))?;
    local
      .spawn_local(async { listen_to_commands(receiver, terminate_tx).await });

    info!("----------------- DQS server started -----------------");
    info!("Config = {:#?}", config);
    info!("-------------------------------------------------------");

    let entry_module = config.state.module.get_entry_module()?;
    let res = tokio::select! {
      res = terminate_rx => {
        res.map(|_| "Terminated by a termination command".to_owned()).map_err(|e| anyhow!("{}", e))

      },
      res = load_and_run_module(runtime, entry_module) => {
        res.map(|_| "Terminated due to event-loop completion".to_owned()).map_err(|e| anyhow!("{}", e))
      }
    };
    info!("DQS server stopped");
    events_tx
      .send(ServerEvents::Terminated(res.into()))?;
    Ok(())
  });

  match r {
    Ok(()) => Ok(()),
    Err(e) => {
      // Note(sp): if the runtime was terminated because of error,
      // need to send server terminated signal
      local.block_on(&rt, async {
        // The server will be terminated when it's dropped, so ignore
        // the send error here since the server might been already dropped
        let _ = events_tx.send(ServerEvents::Terminated(
          Err(anyhow!("Error running DQS server: {}", e)).into(),
        ));
      });
      Err(e)
    }
  }
}

#[allow(dead_code)]
async fn load_and_run_module(
  mut runtime: JsRuntime,
  entry_module: (ModuleSpecifier, Option<ModuleCode>),
) -> Result<()> {
  let mod_id = runtime
    .load_main_module(&entry_module.0, entry_module.1)
    .await?;

  let rx = runtime.mod_evaluate(mod_id);
  runtime.run_event_loop(Default::default()).await?;
  rx.await
}

#[allow(dead_code)]
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
