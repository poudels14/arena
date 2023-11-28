use crate::Result;

use super::KeyValueProvider;

pub trait StorageProvider: Send + Sync {
  fn begin_transaction(&self) -> Result<Box<dyn KeyValueProvider>>;
}
