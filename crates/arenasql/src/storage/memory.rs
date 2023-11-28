use super::{KeyValueProvider, StorageProvider};
use crate::Result;

pub struct MemoryStorageProvider {}

impl Default for MemoryStorageProvider {
  fn default() -> Self {
    Self {}
  }
}

impl StorageProvider for MemoryStorageProvider {
  fn begin_transaction(&self) -> Result<Box<dyn KeyValueProvider>> {
    unimplemented!()
  }
}
