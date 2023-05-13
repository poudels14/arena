pub use self::request::HttpRequest;
use self::resources::{HttpConnection, HttpResponseHandle};
use super::BuiltinExtension;
use crate::resolve_from_root;
use anyhow::{anyhow, Result};
use deno_core::{
  op, ByteString, Extension, OpState, ResourceId, StringOrBuffer,
};
use http::header::HeaderName;
use http::{HeaderValue, Response};
use hyper::Body;
use std::cell::RefCell;
use std::rc::Rc;
mod executor;
mod request;
mod resources;
mod stream;
mod tcp;
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
    .ops(vec![op_http_start::decl(), op_http_send_response::decl()])
    .ops(match &config {
      HttpServerConfig::Stream(_) => vec![
        stream::op_http_accept::decl(),
        stream::op_http_listen::decl(),
      ],
      HttpServerConfig::Tcp(_, _) => {
        vec![tcp::op_http_accept::decl(), tcp::op_http_listen::decl()]
      }
      _ => unimplemented!(),
    })
    .state(move |state| {
      state.put::<HttpServerConfig>(config.clone());
    })
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
      let response_handle = state
        .borrow_mut()
        .resource_table
        .add::<HttpResponseHandle>(HttpResponseHandle { sender: resp });
      return Ok(Some((response_handle, req)));
    }
  }
  Ok(None)
}

#[op]
async fn op_http_send_response(
  state: Rc<RefCell<OpState>>,
  rid: u32,
  status: u16,
  headers: Vec<(ByteString, ByteString)>,
  data: Option<StringOrBuffer>,
) -> Result<()> {
  let handle = state
    .borrow()
    .resource_table
    .get::<HttpResponseHandle>(rid)?;

  let mut response_builder = Response::builder().status(status);
  for header in headers {
    response_builder = response_builder.header(
      HeaderName::from_bytes(&header.0)?,
      HeaderValue::from_bytes(&header.1)?,
    );
  }

  let response = response_builder.body(Body::from(
    <StringOrBuffer as Into<bytes::Bytes>>::into(
      data.unwrap_or(StringOrBuffer::String("".to_owned())),
    )
    .slice(0..),
  ))?;
  handle
    .sender
    .send(response)
    .await
    .map_err(|e| anyhow!("{:?}", e))
}
