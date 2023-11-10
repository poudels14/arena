use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Result;
use deno_core::{op2, OpState};

use super::publisher::Publisher;
use super::Data;
use super::IncomingEvent;
use super::OutgoingEvent;

#[op2(async)]
#[serde]
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
#[op2(async)]
pub async fn op_cloud_pubsub_publish(
  state: Rc<RefCell<OpState>>,
  #[serde] data: Data,
  #[string] path: Option<String>,
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
