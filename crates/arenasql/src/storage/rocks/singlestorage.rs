use std::sync::Arc;

use derivative::Derivative;
use rocksdb::FlushOptions;

use crate::storage::{StorageProvider, Transaction};
use crate::Result as DatabaseResult;

use super::kv::RocksDatabase;

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct RocksStorage {
  kv: Arc<RocksDatabase>,
}

impl RocksStorage {
  pub fn new(path: &str) -> DatabaseResult<Self> {
    let kv = super::kv::open(path)?;
    Ok(Self { kv: Arc::new(kv) })
  }

  pub fn get_db_size(&self) -> DatabaseResult<usize> {
    let live_files = self.kv.live_files()?;
    let total_size = live_files
      .iter()
      .map(|f| {
        println!("file = {:?}", f);
        f.size
      })
      .sum();
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
  fn begin_transaction(&self) -> DatabaseResult<Arc<dyn Transaction>> {
    Ok(Arc::new(super::Transaction::new(self.kv.clone())?))
  }
}
