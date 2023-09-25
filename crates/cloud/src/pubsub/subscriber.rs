use std::sync::Arc;

use derivative::Derivative;
use fastwebsockets::FragmentCollector;
use hyper::upgrade::Upgraded;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use super::filter::EventFilter;
use super::{Node, OutgoingEvent};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Subscriber {
  pub id: String,
  pub node: Node,

  pub out_stream: EventSink,

  pub filter: EventFilter,
  // TODO(sagar): store access of the subscriber and check permission
  // for each message based on workspaceId/appId/path, etc before
  // publishing the message
  // access:
}

#[derive(Derivative)]
#[derivative(Debug)]
pub enum EventSink {
  Websocket(
    #[derivative(Debug = "ignore")] Arc<Mutex<FragmentCollector<Upgraded>>>,
  ),
  Stream(Sender<Vec<OutgoingEvent>>),
}
