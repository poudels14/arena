use std::sync::Arc;

use crate::Result;

pub trait StorageProvider: Send + Sync {
  fn begin_transaction(&self) -> Result<Arc<dyn super::Transaction>>;
}
