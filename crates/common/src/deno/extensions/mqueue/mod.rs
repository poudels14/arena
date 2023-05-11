use anyhow::{anyhow, bail, Result};
use deno_core::error::AnyError;
use deno_core::{op, Extension, OpState, Resource, ResourceId};
use serde::Serialize;
use serde_json::Value;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Serialize)]
struct V8Message {
  rid: ResourceId,
  msg: Value,
}

pub(super) type ResponseSender = mpsc::Sender<Value>;

#[derive(Clone)]
struct MessageQueue {
  pub stream: Rc<RefCell<mpsc::Receiver<(Value, ResponseSender)>>>,
  /// A duration after which Value::Null is sent as the response
  /// This is necessary in cases where the V8 might not respond
  pub response_timeout: Option<Duration>,
}

impl Resource for MessageQueue {
  fn name(&self) -> Cow<str> {
    "messageQueue".into()
  }

  fn close(self: Rc<Self>) {
    // TODO(sagar): do we need to close the message queue
  }
}

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

pub fn init() -> Extension {
  Extension::builder("arena/mqueue")
    .ops(vec![
      self::op_mqueue_listen::decl(),
      self::op_mqueue_send_response::decl(),
    ])
    .build()
}

#[op]
async fn op_mqueue_listen(
  state: Rc<RefCell<OpState>>,
  queue_id: u32,
) -> Result<V8Message, AnyError> {
  let queue = state
    .borrow_mut()
    .resource_table
    .get::<MessageQueue>(queue_id)?;

  // TODO(sagar): send default value (null) to channels
  // that expired

  let mut queue = queue.stream.borrow_mut();
  if let Some((msg, sender)) = queue.recv().await {
    let rid = state
      .borrow_mut()
      .resource_table
      .add(ResponseHandler { sender });

    return Ok(V8Message { msg, rid });
  };

  bail!("failed to get message from mqueue queue");
}

#[op]
async fn op_mqueue_send_response(
  state: Rc<RefCell<OpState>>,
  rid: u32,
  resonse: Value,
) -> Result<()> {
  let handler = state
    .borrow_mut()
    .resource_table
    .get::<ResponseHandler>(rid)?;

  handler
    .sender
    .send(resonse)
    .await
    .map_err(|e| anyhow!("{:}", e))?;

  Ok(())
}
