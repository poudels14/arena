use super::http::{HttpRequest, HttpResponse};
use anyhow::{anyhow, bail, Result};
use bytes::Bytes;
use deno_core::error::AnyError;
use deno_core::{
  op, ByteString, Extension, OpState, Resource, ResourceId, StringOrBuffer,
};
use serde::Serialize;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::sync::mpsc;

#[derive(Serialize)]
struct HttpRequestV8 {
  internal: HttpRequest,
  rid: ResourceId,
}

pub(super) type ResponseSender = mpsc::Sender<HttpResponse>;

#[derive(Clone)]
struct RequestReceiver(
  Rc<RefCell<mpsc::Receiver<(HttpRequest, ResponseSender)>>>,
);

#[derive(Clone, Debug)]
struct ResponseHandler {
  sender: ResponseSender,
}

impl Resource for ResponseHandler {
  fn name(&self) -> Cow<str> {
    "responseHandler".into()
  }

  fn close(self: Rc<Self>) {
    // TODO(sagar): do we need to close sender?
  }
}

pub fn init(
  requests_receiver: Rc<RefCell<mpsc::Receiver<(HttpRequest, ResponseSender)>>>,
) -> Extension {
  Extension::builder("arena/workspace-server/ext")
    .ops(vec![
      self::op_receive_request::decl(),
      self::op_send_response::decl(),
    ])
    .state(move |state| {
      state
        .put::<RequestReceiver>(RequestReceiver(requests_receiver.to_owned()));
    })
    .build()
}

#[op]
async fn op_receive_request(
  state: Rc<RefCell<OpState>>,
) -> Result<HttpRequestV8, AnyError> {
  let receiver = state.borrow().borrow::<RequestReceiver>().clone();

  let mut receiver = receiver.0.borrow_mut();
  if let Some((req, res)) = receiver.recv().await {
    let rid = state
      .borrow_mut()
      .resource_table
      .add(ResponseHandler { sender: res });

    return Ok(HttpRequestV8 { internal: req, rid });
  };

  bail!("Something went wrong");
}

#[op]
async fn op_send_response(
  state: Rc<RefCell<OpState>>,
  rid: u32,
  status: u16,
  headers: Vec<(ByteString, ByteString)>,
  data: Option<StringOrBuffer>,
) -> Result<()> {
  let handler = state
    .borrow_mut()
    .resource_table
    .get::<ResponseHandler>(rid)?;

  handler
    .sender
    .send(HttpResponse {
      status,
      headers,
      body: match data {
        Some(v) => Some(
          std::str::from_utf8(&Bytes::from(v))
            .map_err(|e| anyhow!("{:}", e))?
            .to_owned(),
        ),
        None => None,
      },
      close: true,
    })
    .await
    .map_err(|e| anyhow!("{:}", e))?;

  Ok(())
}
