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

  fn get_or_log_error(&self, key: &[u8]) -> Option<Vec<u8>> {
    self.get(key).unwrap_or_else(|e| {
      tracing::error!("Error loading key-value from storage: {:?}", e);
      None
    })
  }

  fn get_for_update(
    &self,
    key: &[u8],
    exclusive: bool,
  ) -> Result<Option<Vec<u8>>>;

  fn scan(&self, prefix: &[u8]) -> Result<Vec<(Box<[u8]>, Box<[u8]>)>>;

  fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
    self.put_all(&vec![(key, value)])
  }

  fn put_all(&self, rows: &[(&[u8], &[u8])]) -> Result<()>;

  fn commit(&self) -> Result<()>;

  fn rollback(&self) -> Result<()>;

  fn done(&self) -> Result<bool>;
}
