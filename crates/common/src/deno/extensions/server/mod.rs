pub use self::request::HttpRequest;
use self::resources::{HttpConnection, HttpResponseHandle};
use self::response::ParsedHttpResponse;
use super::BuiltinExtension;
use crate::deno::extensions::server::websocket::WebsocketStream;
use crate::resolve_from_root;
use anyhow::{anyhow, bail, Result};
use deno_core::{
  op, ByteString, Extension, OpState, ResourceId, StringOrBuffer,
};
use std::cell::RefCell;
use std::rc::Rc;
use tokio::sync::oneshot;
pub mod errors;
mod executor;
pub mod request;
mod resources;
pub mod response;
mod stream;
mod tcp;
mod websocket;
pub use resources::HttpServerConfig;

pub fn extension(config: HttpServerConfig) -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(self::init(config)),
    runtime_modules: vec![],
    snapshot_modules: vec![(
      "@arena/runtime/server",
      resolve_from_root!("../../js/arena-runtime/dist/server.js"),
    )],
  }
}

/// initialize server extension with given (address, port)
fn init(config: HttpServerConfig) -> Extension {
  Extension::builder("arena/runtime/server")
    .ops(vec![
      op_http_start::decl(),
      op_http_send_response::decl(),
      websocket::op_websocket_recv::decl(),
      websocket::op_websocket_send::decl(),
    ])
    .ops(match &config {
      HttpServerConfig::Stream(_) => vec![
        stream::op_http_accept::decl(),
        stream::op_http_listen::decl(),
      ],
      HttpServerConfig::Tcp {
        address: _,
        port: _,
        serve_dir: _,
      } => {
        vec![tcp::op_http_accept::decl(), tcp::op_http_listen::decl()]
      }
      _ => unimplemented!(),
    })
    .state(move |state| {
      state.put::<HttpServerConfig>(config.clone());
    })
    .force_op_registration()
    .build()
}

#[op]
async fn op_http_start(
  state: Rc<RefCell<OpState>>,
  rid: u32,
) -> Result<Option<(ResourceId, HttpRequest)>> {
  let connection = state.borrow().resource_table.get::<HttpConnection>(rid)?;
  let stream = connection.req_stream.try_borrow_mut();

  // Note(sagar): if the stream is already borrowed, that means it's already
  // being listened to; Since, the stream can only be listened to once,
  // return Ok(None)
  if let Ok(mut rx) = stream {
    if let Some((req, resp)) = rx.recv().await {
      let response_handle =
        state.borrow_mut().resource_table.add::<HttpResponseHandle>(
          HttpResponseHandle(RefCell::new(Some(resp))),
        );
      return Ok(Some((response_handle, req)));
    }
  }
  Ok(None)
}

/// This sends a response
/// If the connection is upgraded to websocket, this returns a resource id
/// of a (mpsc::Receiver, mpsc::Sender) to receive and send websocket messages
/// respectively
#[op]
async fn op_http_send_response(
  state: Rc<RefCell<OpState>>,
  rid: u32,
  status: u16,
  headers: Vec<(ByteString, ByteString)>,
  data: Option<StringOrBuffer>,
) -> Result<Option<(ResourceId, ResourceId, Option<StringOrBuffer>)>> {
  let handle = {
    state
      .borrow_mut()
      .resource_table
      .get::<HttpResponseHandle>(rid)?
  };

  let mut res = ParsedHttpResponse {
    rid,
    status,
    headers,
    data,
    websocket_tx: None,
  };

  let mut data = None;
  let websocket_rx = match res.has_upgrade_header() {
    true => {
      data = res.data.take();

      let c = oneshot::channel::<WebsocketStream>();
      res.websocket_tx = Some(c.0);
      Some(c.1)
    }
    false => None,
  };

  let sender = handle.0.take();
  if let Some(sender) = sender {
    match sender.send(res).map_err(|e| anyhow!("{:?}", e)) {
      Ok(_) => {
        if let Some(rx) = websocket_rx {
          let websocket = rx.await?;
          let resource_table = &mut state.borrow_mut().resource_table;
          let rx_id = resource_table.add(websocket.rx);
          let tx_id = resource_table.add(websocket.tx);
          return Ok(Some((rx_id, tx_id, data)));
        }
        return Ok(None);
      }
      Err(e) => bail!("{}", e),
    }
  }
  bail!("Error sending response");
}
