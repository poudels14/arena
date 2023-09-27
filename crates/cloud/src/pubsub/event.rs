use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::identity::Identity;

#[derive(Debug, Default)]
pub struct EventBuffer {
  /// Event buffer
  // TODO(sagar): might have to create buffer for each path
  // since not all subscribers will have access to all events
  // and state/changeset will be tricky to manage
  pub buffer: Vec<OutgoingEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutgoingEvent {
  pub source: Identity,
  /// Setting the path when publishing the event will allow ACL checker
  /// to check whether the user has access to the path before sending
  /// the event with that path
  pub path: Option<String>,
  pub data: Data,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingEvent {
  pub source: Identity,
  pub path: String,
  pub message: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Data {
  Message(Value),
  State(Value),
  #[serde(rename_all = "camelCase")]
  ChangeSet {
    /// Reference id of the state that this changeset is based on
    reference_id: String,
    /// Sequence number of the changeset. It starts with 1
    seq_id: u64,
    delta: Value,
  },
}
