use super::events::{ServerEvent, ServerEvents};
use super::handle::{ClientRequest, ServerHandle, ServerRequest};
use crate::Workspace;
use anyhow::{anyhow, bail, Result};
use derivative::Derivative;
use serde_json::Value;
use tokio::sync::oneshot;

#[derive(Derivative)]
#[derivative(Clone, Debug, Default)]
pub struct ServerOptions {
  /// Whether to run the server in dev mode
  pub dev_mode: bool,

  /// Socket address of the workspace
  /// Defaults to 0.0.0.0
  #[derivative(Default(value = "String::from(\"0.0.0.0\")"))]
  pub address: String,

  /// The port workspace server is listening to
  /// Defaults to a random port
  #[derivative(Default(value = "0"))]
  pub port: u16,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct WorkspaceServerHandle {
  /// Socket address of the workspace
  pub address: String,

  /// The port workspace server is listening to
  pub port: u16,

  /// Workspace config
  pub(crate) workspace: Workspace,

  /// Message handle that can be used to communicate with the server
  pub(crate) handle: ServerHandle<ServerRequest, ClientRequest>,

  /// Handle for events emitted by the server
  pub(crate) events: ServerEvents,
}

impl WorkspaceServerHandle {
  /// Wait for the server to shutdown
  pub async fn wait_for_termination(&self) -> Result<()> {
    self
      .events
      .wait_until(ServerEvent::Terminated)
      .await
      .and_then(|_| Ok(()))
  }

  pub async fn terminate(&self) -> Result<()> {
    Ok(())
  }

  /// Sends a message to this WorkspaceServer
  pub async fn send_request(&self, msg: ClientRequest) -> Result<Value> {
    let (resp_tx, resp_rx) = oneshot::channel::<Value>();
    self.handle.server.tx.send((msg, resp_tx)).await.unwrap();
    resp_rx.await.map_err(|e| anyhow!(e))
  }

  /// Pipes Http Request to the workspace server
  /// This is helpful if we want to send a request from
  /// custom server to WorkspaceServer
  pub async fn pipe_request(&self) -> Result<()> {
    bail!("not implemented");
  }
}
