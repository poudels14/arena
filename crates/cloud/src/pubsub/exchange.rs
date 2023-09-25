use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Result;
use derivative::Derivative;
use fastwebsockets::{Frame, Payload};
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::debug;
use uuid::Uuid;

use super::publisher::{Publisher, PublisherHandle};
use super::{EventSink, Node, OutgoingEvent, Subscriber};

#[derive(Derivative)]
#[derivative(Clone, Debug)]
/// Each exchange will be associated with a workspace
/// For now, drive all the message pub/sub of the given exchange
/// in a single threaded tokio runtime
pub struct Exchange {
  pub workspace_id: String,

  port: mpsc::Sender<OutgoingEvent>,
  stream: Arc<Mutex<mpsc::Receiver<OutgoingEvent>>>,

  subscribers: Arc<RwLock<BTreeMap<String, Subscriber>>>,
  publishers: Arc<RwLock<BTreeMap<String, PublisherHandle>>>,
}

impl Exchange {
  pub fn new(workspace_id: String) -> Self {
    let (tx, rx) = mpsc::channel(200);

    Self {
      workspace_id,
      subscribers: Arc::new(RwLock::new(BTreeMap::new())),
      publishers: Arc::new(RwLock::new(BTreeMap::new())),
      port: tx,
      stream: Arc::new(Mutex::new(rx)),
    }
  }

  pub async fn add_subscriber(&self, subscriber: Subscriber) -> Result<()> {
    let mut subs = self.subscribers.write().await;

    // TODO(sagar): flush bufferred events to this new subscriber
    subs.insert(subscriber.id.clone(), subscriber);
    Ok(())
  }

  pub async fn new_publisher(&self, source: Node) -> Publisher {
    let mut publishers = self.publishers.write().await;
    let publisher_id = Uuid::new_v4().to_string();

    let (tx, rx) = mpsc::channel(20);
    let handle = PublisherHandle {
      id: publisher_id.to_string(),
      source: source.clone(),
      stream: Arc::new(Mutex::new(tx)),
      buffer: Arc::new(Mutex::new(Default::default())),
    };

    publishers.insert(publisher_id.to_string(), handle);

    Publisher {
      id: publisher_id,
      source,
      in_stream: Rc::new(RefCell::new(rx)),
      out_stream: self.port.clone(),
    }
  }

  // TODO(sagar): run this in a dedicated thread with a local event pool?
  pub async fn run(&self) -> Result<()> {
    let stream = self.stream.clone();
    let subscribers = self.subscribers.clone();
    tokio::task::spawn(async move {
      let mut rx = stream.lock().await;

      // TODO(sagar): instead of sending single event, buffer events for
      // X milliseconds and send it?
      while let Some(event) = rx.recv().await {
        let mut unsubscribed_subs = vec![];
        let subs = subscribers.read().await;
        for (sub_id, sub) in subs.iter() {
          match &sub.out_stream {
            EventSink::Stream(st) => {
              let _ = st.send(vec![event.clone()]).await;
            }
            EventSink::Websocket(fc) => {
              let mut fc = fc.lock().await;
              let r = fc
                .write_frame(Frame::text(Payload::Owned(
                  serde_json::to_vec(&vec![&event]).unwrap(),
                )))
                .await;

              match r {
                Err(e) => {
                  unsubscribed_subs.push(sub_id.clone());
                  tracing::error!(
                    "Error sending data to a subscriber [{:?}]: {:?}",
                    sub.node,
                    e
                  );
                }
                _ => {}
              }
            }
          }
        }
        drop(subs);

        if !unsubscribed_subs.is_empty() {
          let mut subs = subscribers.write().await;
          unsubscribed_subs.iter().for_each(|id| {
            subs.remove(id);
          });
          debug!("Removed pubsub subscribers: {:?}", unsubscribed_subs);
        }
      }
    });
    Ok(())
  }
}
