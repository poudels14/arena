use super::{KeyValueIterator, KeyValueStore, KeyValueStoreProvider};
use crate::Result;

pub struct MemoryKeyValueStoreProvider {}

impl Default for MemoryKeyValueStoreProvider {
  fn default() -> Self {
    Self {}
  }
}

impl KeyValueStoreProvider for MemoryKeyValueStoreProvider {
  fn new_transaction(&self) -> Result<Box<dyn KeyValueStore>> {
    Ok(Box::new(MemoryKeyValueStore {}))
  }
}

pub struct MemoryKeyValueStore {}

impl KeyValueStore for MemoryKeyValueStore {
  fn atomic_update(
    &self,
    _group: super::KeyValueGroup,
    _key: &[u8],
    _updater: &dyn Fn(Option<Vec<u8>>) -> Result<Vec<u8>>,
  ) -> Result<Vec<u8>> {
    unimplemented!()
  }

  fn get_for_update(
    &self,
    _group: super::KeyValueGroup,
    _key: &[u8],
    _exclusive: bool,
  ) -> Result<Option<Vec<u8>>> {
    Ok(None)
  }

  fn get(
    &self,
    _group: super::KeyValueGroup,
    _key: &[u8],
  ) -> Result<Option<Vec<u8>>> {
    Ok(None)
  }

  fn scan_with_prefix(
    &self,
    _group: super::KeyValueGroup,
    _prefix: &[u8],
  ) -> Result<Box<dyn super::KeyValueIterator>> {
    Ok(Box::new(EmptyIterator {}))
  }

  fn put(
    &self,
    _group: super::KeyValueGroup,
    _key: &[u8],
    _value: &[u8],
  ) -> Result<()> {
    unimplemented!()
  }

  fn put_all(
    &self,
    _group: super::KeyValueGroup,
    _rows: &[(&[u8], &[u8])],
  ) -> Result<()> {
    unimplemented!()
  }

  fn delete(&self, _group: super::KeyValueGroup, _key: &[u8]) -> Result<()> {
    unimplemented!()
  }

  fn commit(&self) -> Result<()> {
    Ok(())
  }

  fn rollback(&self) -> Result<()> {
    Ok(())
  }
}

pub struct EmptyIterator {}

impl KeyValueIterator for EmptyIterator {
  fn key(&self) -> Option<&[u8]> {
    None
  }

  fn get(&self) -> Option<(&[u8], &[u8])> {
    None
  }

  fn next(&mut self) {}
}
