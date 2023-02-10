use anyhow::{bail, Result};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;

/// These are events emitted by server
/// These events can be used to determine the state of the server
#[derive(Clone, Debug, PartialEq)]
pub enum ServerEvent {
  Started,
  Terminated,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerStarted {
  pub address: String,
  pub port: u16,
}

#[derive(Derivative)]
#[derivative(Clone, Debug)]
pub struct ServerEvents {
  #[derivative(Debug = "ignore")]
  pub(crate) sender: Sender<(ServerEvent, Value)>,
}

impl ServerEvents {
  pub(crate) fn new() -> Self {
    let (s, _) = broadcast::channel(100);
    Self { sender: s }
  }

  /// Execute a function when the given event type is received
  #[allow(dead_code)]
  pub async fn on(&self, _event: ServerEvent) -> Result<()> {
    bail!("not impl");
  }

  /// Wait until an event of the given type is received
  pub async fn wait_until(&self, event: ServerEvent) -> Result<Value> {
    let mut rx = self.sender.subscribe();
    loop {
      let (e, data) = rx.recv().await?;
      if event == e {
        return Ok(data);
      }
    }
  }
}
