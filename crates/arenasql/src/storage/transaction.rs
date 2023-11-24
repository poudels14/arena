use crate::Result;

pub trait Transaction: Send + Sync {
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

  fn put(&self, key: &[u8], value: &[u8]) -> Result<()>;

  fn put_all(&self, rows: &[(&[u8], &[u8])]) -> Result<()>;

  fn commit(&self) -> Result<()>;

  fn rollback(&self) -> Result<()>;

  fn done(&self) -> Result<bool>;
}
