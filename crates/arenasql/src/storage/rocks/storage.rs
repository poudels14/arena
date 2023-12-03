use std::path::Path;
use std::sync::Arc;

use derivative::Derivative;
pub use rocksdb::Cache;
use rocksdb::{
  ColumnFamilyDescriptor, DBCompressionType, FlushOptions, MultiThreaded,
  OptimisticTransactionDB, Options as RocksOptions,
};

use super::KeyValueProvider;
use crate::storage::{self, KeyValueGroup, StorageProvider};
use crate::Result as DatabaseResult;

pub(super) type RocksDatabase = OptimisticTransactionDB<MultiThreaded>;

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct RocksStorage {
  kv: Arc<RocksDatabase>,
}

impl RocksStorage {
  pub fn new<P: AsRef<Path>>(path: P) -> DatabaseResult<Self> {
    Self::new_with_cache(path, None)
  }

  pub fn new_with_cache<P: AsRef<Path>>(
    path: P,
    cache: Option<Cache>,
  ) -> DatabaseResult<Self> {
    let mut opts = RocksOptions::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);
    opts.set_compression_type(DBCompressionType::Lz4);
    opts.set_max_background_jobs(0);

    // enable blob files
    opts.set_enable_blob_files(true);
    // TODO: set min blob size so that vector embeddings aren't stored in
    // blobs but documents are
    opts.set_enable_blob_gc(true);
    // this isn't neessary in WAL mode but set it anyways
    opts.set_atomic_flush(true);
    if let Some(cache) = cache {
      opts.set_row_cache(&cache);
    }

    let kv: RocksDatabase = OptimisticTransactionDB::open_cf_descriptors(
      &opts,
      path,
      vec![
        ColumnFamilyDescriptor::new(
          KeyValueGroup::Locks.to_string(),
          RocksOptions::default(),
        ),
        ColumnFamilyDescriptor::new(
          KeyValueGroup::Schemas.to_string(),
          RocksOptions::default(),
        ),
        ColumnFamilyDescriptor::new(
          KeyValueGroup::Indexes.to_string(),
          RocksOptions::default(),
        ),
        ColumnFamilyDescriptor::new(
          KeyValueGroup::Rows.to_string(),
          RocksOptions::default(),
        ),
      ],
    )?;
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
