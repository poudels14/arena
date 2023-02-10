use derivative::Derivative;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};

/// A request that a workspace server can send to the parent process
/// It's called ServerRequest because the request orignated from the server
// TODO(sagar): we might be able to get rid of this and just use Server events
#[derive(Clone, Debug)]
pub enum ServerRequest {
  /// Send metrics to the client
  Metrics,
}

/// A request that client can send to server
/// It's called ClientRequest because the request originated from the client
#[derive(Clone, Debug)]
pub enum ClientRequest {
  /// Shutdown server
  Shutdown,
}

/// This is used to communicate with a WorkspaceServer running on it's own
/// thread
#[derive(Clone, Debug)]
pub(crate) struct ServerHandle<Cr, Sr> {
  /// A handle that server uses to receive request from the parent process or
  /// send request to the parent process
  /// Note that ServerRequest is sent using this handle
  pub client: Handle<Sr, Cr, Value>,

  /// A handle that client uses to receive reqiest from the server process or
  /// send request to the server process running in a separate thread
  /// Note that ClientRequest is sent using this handle
  pub server: Handle<Cr, Sr, Value>,
}

impl ServerHandle<ServerRequest, ClientRequest> {
  pub fn new() -> Self {
    let (c_tx, c_rx) =
      mpsc::channel::<(ServerRequest, oneshot::Sender<Value>)>(2);
    let (s_tx, s_rx) =
      mpsc::channel::<(ClientRequest, oneshot::Sender<Value>)>(2);

    Self {
      client: Handle::new(s_rx, c_tx),
      server: Handle::new(c_rx, s_tx),
    }
  }
}

#[derive(Derivative)]
#[derivative(Clone, Debug)]
pub struct Handle<In, Out, Res> {
  #[derivative(Debug = "ignore")]
  pub(crate) rx: Arc<Mutex<mpsc::Receiver<(In, oneshot::Sender<Res>)>>>,

  #[derivative(Debug = "ignore")]
  pub(crate) tx: mpsc::Sender<(Out, oneshot::Sender<Res>)>,
}

impl<In, Out, Res> Handle<In, Out, Res> {
  pub fn new(
    rx: mpsc::Receiver<(In, oneshot::Sender<Res>)>,
    tx: mpsc::Sender<(Out, oneshot::Sender<Res>)>,
  ) -> Self {
    Self {
      rx: Arc::new(Mutex::new(rx)),
      tx: tx,
    }
  }
}
