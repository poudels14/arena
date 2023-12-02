use crate::Result;

pub trait RawIterator {
  fn key(&self) -> Option<&[u8]>;

  fn value(&self) -> Option<&[u8]>;

  fn get(&self) -> Option<(&[u8], &[u8])>;

  fn next(&mut self);
}

/// This is the interface to write key/values to the database.
/// The implementation of this trait doesn't have to be thread
/// safe since the transaction manager ensures the thread safety.
pub trait KeyValueProvider {
  /// Update the value of the given key atomically.
  /// This should return error if the key was modified
  /// by another transaction after the value was read first by
  /// this transaction.
  fn atomic_update(
    &self,
    key: &[u8],
    updater: &dyn Fn(Option<Vec<u8>>) -> Vec<u8>,
  ) -> Result<Vec<u8>>;

  fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

  fn get_for_update(
    &self,
    key: &[u8],
    exclusive: bool,
  ) -> Result<Option<Vec<u8>>>;

  fn scan(&self, prefix: &[u8]) -> Result<Vec<(Box<[u8]>, Box<[u8]>)>>;

  fn scan_raw(&self, _prefix: &[u8]) -> Result<Box<dyn RawIterator>> {
    unimplemented!()
  }

  fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
    self.put_all(&vec![(key, value)])
  }

  fn put_all(&self, rows: &[(&[u8], &[u8])]) -> Result<()>;

  fn commit(&self) -> Result<()>;

  fn rollback(&self) -> Result<()>;
}
