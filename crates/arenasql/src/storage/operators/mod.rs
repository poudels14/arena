mod rowid;
mod rows;
mod table;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::transaction::TransactionState;
use super::{KeyValueGroup, KeyValueProvider, Serializer};

/// Uses interior mutability to store the KeyValue provider trait
/// because owned reference to the trait is required in order to
/// commit the transaction
pub struct StorageOperator {
  pub(crate) kv: Arc<Box<dyn KeyValueProvider>>,
  pub(crate) serializer: Serializer,
  pub(crate) lock: Arc<AtomicUsize>,
}

impl StorageOperator {
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

impl Drop for StorageOperator {
  fn drop(&mut self) {
    let _ = self.lock.compare_exchange(
      TransactionState::Locked as usize,
      TransactionState::Free as usize,
      Ordering::Acquire,
      Ordering::Relaxed,
    );
  }
}
