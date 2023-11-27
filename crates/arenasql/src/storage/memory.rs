use std::sync::Arc;

use super::StorageProvider;
use crate::Result;

pub struct MemoryStorageProvider {}

impl Default for MemoryStorageProvider {
  fn default() -> Self {
    Self {}
  }
}

impl StorageProvider for MemoryStorageProvider {
  fn begin_transaction(&self) -> Result<Arc<dyn super::Transaction>> {
    unimplemented!()
  }
}
