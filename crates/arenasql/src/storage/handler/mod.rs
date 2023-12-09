mod indexes;
mod rowid;
mod rows;
mod table;

use std::sync::Arc;

use super::transaction::TransactionState;
use super::{KeyValueGroup, KeyValueStore, Serializer};

/// Uses interior mutability to store the KeyValue provider trait
/// because owned reference to the trait is required in order to
/// commit the transaction
pub struct StorageHandler {
  pub(crate) kv: Arc<Box<dyn KeyValueStore>>,
  pub(crate) serializer: Serializer,
  #[allow(unused)]
  pub(crate) transaction_state: Option<Arc<TransactionState>>,
}

impl StorageHandler {
  pub fn get_or_log_error(
    &self,
    group: KeyValueGroup,
    key: &[u8],
  ) -> Option<Vec<u8>> {
    self.kv.get(group, key).unwrap_or_else(|e| {
      eprintln!("Error loading data from KeyValue store: {:?}", e);
      None
    })
  }
}

impl Drop for StorageHandler {
  fn drop(&mut self) {
    if let Some(state) = &self.transaction_state {
      state.unlock();
    }
  }
}
