mod executor;
mod resources;
mod stream;
mod tcp;
mod websocket;

pub mod errors;
pub mod request;
pub mod response;
mod sse;

use bytes::BytesMut;
pub use request::HttpRequest;
pub use resources::HttpServerConfig;

use std::cell::RefCell;
use std::rc::Rc;

use anyhow::{anyhow, bail, Result};
use deno_core::{
  op2, ByteString, Extension, Op, OpState, Resource, ResourceId, StringOrBuffer,
};
use tokio::sync::oneshot;

use self::resources::{HttpConnection, HttpResponseHandle};
use self::response::{ParsedHttpResponse, StreamResponseWriter, StreamType};
use self::sse::{Event, EventFlags};
use super::r#macro::js_dist;
use super::BuiltinExtension;
use crate::extensions::server::websocket::WebsocketStream;

pub fn extension(config: HttpServerConfig) -> BuiltinExtension {
  BuiltinExtension::new(
    Some(self::init(config)),
    vec![("@arena/runtime/server", js_dist!("/server.js"))],
  )
}

/// initialize server extension with given (address, port)
fn init(config: HttpServerConfig) -> Extension {
  let ops = match &config {
    HttpServerConfig::Stream(_) => {
      vec![stream::op_http_accept::DECL, stream::op_http_listen::DECL]
    }
    HttpServerConfig::Tcp {
      address: _,
      port: _,
      serve_dir: _,
    } => {
      vec![tcp::op_http_accept::DECL, tcp::op_http_listen::DECL]
    }
    _ => unimplemented!(),
  };
  Extension {
    name: "arena/runtime/server",
    ops: vec![
      vec![
        op_http_start::DECL,
        op_http_send_response::DECL,
        op_http_write_text_to_stream::DECL,
        op_http_write_uint8_event_to_stream::DECL,
        op_http_write_bytes_to_stream::DECL,
        op_http_close_stream::DECL,
        websocket::op_websocket_recv::DECL,
        websocket::op_websocket_send::DECL,
      ],
      ops,
    ]
    .concat()
    .into(),
    op_state_fn: Some(Box::new(move |state: &mut OpState| {
      state.put::<HttpServerConfig>(config.clone());
    })),
    enabled: true,
    ..Default::default()
  }
}

#[op2(async)]
#[serde]
async fn op_http_start(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
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
/// If the connection is upgraded to websocket, this returns a tuple of
/// resource id of a mpsc::Receiver, mpsc::Sender, and data to receive
/// and send websocket messages
/// It returns a tuple of (resource_id, null, null) if stream option is
/// true and data can be written to the returned resource id using
/// `op_http_write_to_stream`
#[op2(async)]
#[serde]
async fn op_http_send_response(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
  #[smi] status: u16,
  #[serde] headers: Vec<(ByteString, ByteString)>,
  #[serde] data: Option<StringOrBuffer>,
  #[serde] stream: Option<StreamType>,
) -> Result<Option<(ResourceId, Option<ResourceId>, Option<StringOrBuffer>)>> {
  let handle = {
    state
      .borrow_mut()
      .resource_table
      .get::<HttpResponseHandle>(rid)?
  };

  let (stream, writer_id) = match stream {
    Some(r#type) => {
      let (writer, reader) = response::channel(r#type);
      let writer_id = state
        .borrow_mut()
        .resource_table
        .add::<StreamResponseWriter>(writer);
      (Some(reader), Some(writer_id))
    }
    _ => (None, None),
  };

  let mut res = ParsedHttpResponse {
    rid,
    status,
    headers,
    data,
    stream,
    ..Default::default()
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
          return Ok(Some((rx_id, Some(tx_id), data)));
        }
        if let Some(writer_id) = writer_id {
          return Ok(Some((writer_id, None, None)));
        }
        return Ok(None);
      }
      Err(e) => bail!("{}", e),
    }
  }
  bail!("Error sending response");
}

/// Write textual data to the given stream and returns the length
/// of the bytes written.
/// if it failed to write (probably because the stream is already closed),
/// returns -1
#[op2(async)]
async fn op_http_write_text_to_stream(
  state: Rc<RefCell<OpState>>,
  #[smi] writer_id: ResourceId,
  #[string] data: String,
) -> Result<i32> {
  let writer = {
    state
      .borrow()
      .resource_table
      .get::<StreamResponseWriter>(writer_id)?
  };

  let len = data.len();
  match writer
    .write_event(Event {
      buffer: BytesMut::from(&data.as_bytes()[..]),
      flags: EventFlags::HAS_DATA,
    })
    .await
  {
    Ok(_) => return Ok(len.try_into().unwrap()),
    // If there's any error writing to the stream, close the stream resource
    // and return error
    Err(_) => {
      state
        .borrow_mut()
        .resource_table
        .take::<StreamResponseWriter>(writer_id)?
        .close();
      return Ok(-1);
    }
  }
}

#[op2(async)]
async fn op_http_write_uint8_event_to_stream(
  state: Rc<RefCell<OpState>>,
  #[smi] writer_id: ResourceId,
  #[buffer(copy)] data: Vec<u8>,
) -> Result<i32> {
  let writer = {
    state
      .borrow()
      .resource_table
      .get::<StreamResponseWriter>(writer_id)?
  };

  let len = data.len();
  match writer
    .write_event(Event {
      buffer: BytesMut::from(&data[..]),
      flags: EventFlags::HAS_EVENT,
    })
    .await
  {
    Ok(_) => return Ok(len.try_into().unwrap()),
    // If there's any error writing to the stream, close the stream resource
    // and return error
    Err(_) => {
      state
        .borrow_mut()
        .resource_table
        .take::<StreamResponseWriter>(writer_id)?
        .close();
      return Ok(-1);
    }
  }
}

/// Write buffer to the given stream and returns the length of bytes written
/// if it failed to write (probably because the stream is already closed),
/// returns -1
#[op2(async)]
async fn op_http_write_bytes_to_stream(
  state: Rc<RefCell<OpState>>,
  #[smi] writer_id: ResourceId,
  #[buffer(copy)] bytes: bytes::Bytes,
) -> Result<i32> {
  let writer = {
    state
      .borrow()
      .resource_table
      .get::<StreamResponseWriter>(writer_id)?
  };

  let len = bytes.len();
  match writer.write_bytes(Ok(bytes)).await {
    Ok(_) => return Ok(len.try_into().unwrap()),
    // If there's any error writing to the stream, close the stream resource
    // and return error
    Err(_) => {
      state
        .borrow_mut()
        .resource_table
        .take::<StreamResponseWriter>(writer_id)?
        .close();
      return Ok(-1);
    }
  }
}

/// Return true if stream closed successful
#[op2(fast)]
fn op_http_close_stream(
  state: &mut OpState,
  #[smi] writer_id: ResourceId,
) -> bool {
  state
    .resource_table
    .take::<StreamResponseWriter>(writer_id)
    .map(|r| {
      r.close();
      true
    })
    .unwrap_or(false)
}
