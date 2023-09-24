use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use deno_core::Resource;
use derivative::Derivative;
use tokio::sync::{mpsc, Mutex};

use super::{EventBuffer, IncomingEvent, Node, OutgoingEvent};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct PublisherHandle {
  pub id: usize,

  pub source: Node,

  pub stream: Arc<Mutex<mpsc::Sender<IncomingEvent>>>,

  /// Event buffer
  pub buffer: Arc<Mutex<EventBuffer>>,
}

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct Publisher {
  pub id: usize,
  pub in_stream: Rc<RefCell<mpsc::Receiver<IncomingEvent>>>,
  pub out_stream: mpsc::Sender<OutgoingEvent>,
}

impl Resource for Publisher {}
