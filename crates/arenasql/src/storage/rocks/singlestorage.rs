use std::sync::Arc;

use derivative::Derivative;
pub use rocksdb::Cache;
use rocksdb::FlushOptions;

use super::kv::RocksDatabase;
use super::KeyValueProvider;
use crate::storage::{self, StorageProvider};
use crate::Result as DatabaseResult;

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct RocksStorage {
  kv: Arc<RocksDatabase>,
}

impl RocksStorage {
  pub fn new(path: &str) -> DatabaseResult<Self> {
    Self::new_with_cache(path, None)
  }

  pub fn new_with_cache(
    path: &str,
    cache: Option<Cache>,
  ) -> DatabaseResult<Self> {
    let kv = super::kv::open(path, cache)?;
    Ok(Self { kv: Arc::new(kv) })
  }

  pub fn get_db_size(&self) -> DatabaseResult<usize> {
    let live_files = self.kv.live_files()?;
    let total_size = live_files.iter().map(|f| f.size).sum();
    Ok(total_size)
  }

  pub fn compact_and_flush(&self) -> DatabaseResult<()> {
    let kv = &self.kv;
    kv.compact_range(None::<&[u8]>, None::<&[u8]>);

    let mut flush_opt = FlushOptions::default();
    flush_opt.set_wait(true);
    kv.flush()?;
    Ok(())
  }
}

impl StorageProvider for RocksStorage {
  fn begin_transaction(
    &self,
  ) -> DatabaseResult<Box<dyn storage::KeyValueProvider>> {
    Ok(Box::new(KeyValueProvider::new(self.kv.clone())?))
  }
}
