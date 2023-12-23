use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use derivative::Derivative;
use runtime::deno::core::Resource;
use tokio::sync::{mpsc, Mutex};

use super::{EventBuffer, IncomingEvent, OutgoingEvent};
use crate::identity::Identity;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct PublisherHandle {
  pub id: String,

  pub source: Identity,

  pub stream: Arc<Mutex<mpsc::Sender<IncomingEvent>>>,

  /// Event buffer
  pub buffer: Arc<Mutex<EventBuffer>>,
}

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct Publisher {
  pub id: String,
  pub source: Identity,
  pub in_stream: Rc<RefCell<mpsc::Receiver<IncomingEvent>>>,
  pub out_stream: mpsc::Sender<OutgoingEvent>,
}

impl Resource for Publisher {}
