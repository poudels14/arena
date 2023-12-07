use super::{KeyValueStore, KeyValueStoreProvider};
use crate::Result;

pub struct MemoryKeyValueStoreProvider {}

impl Default for MemoryKeyValueStoreProvider {
  fn default() -> Self {
    Self {}
  }
}

impl KeyValueStoreProvider for MemoryKeyValueStoreProvider {
  fn new_transaction(&self) -> Result<Box<dyn KeyValueStore>> {
    unimplemented!()
  }
}
