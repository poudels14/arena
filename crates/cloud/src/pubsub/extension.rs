use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Result;
use deno_core::op;
use deno_core::OpState;

use super::publisher::Publisher;
use super::Data;
use super::IncomingEvent;
use super::OutgoingEvent;

#[op]
pub async fn op_cloud_pubsub_subscribe(
  state: Rc<RefCell<OpState>>,
) -> Result<Option<IncomingEvent>> {
  let stream = {
    let state = state.borrow();
    let publisher = state.try_borrow::<Publisher>();
    publisher.map(|p| p.in_stream.clone())
  };

  if let Some(rx) = stream {
    let mut rx = rx.borrow_mut();
    return Ok(rx.recv().await);
  }

  Ok(None)
}

/// Returns boolean indicating whether the data was published
#[op]
pub async fn op_cloud_pubsub_publish(
  state: Rc<RefCell<OpState>>,
  data: Data,
  path: Option<String>,
) -> Result<bool> {
  let state = state.borrow();
  let publisher = state.try_borrow::<Publisher>();
  if let Some(publisher) = publisher {
    publisher
      .out_stream
      .send(OutgoingEvent {
        source: publisher.source.clone(),
        path,
        data,
      })
      .await?;
    return Ok(true);
  }

  Ok(false)
}
