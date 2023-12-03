use std::path::PathBuf;
use std::sync::Arc;

use derivative::Derivative;
pub use rocksdb::Cache;
use rocksdb::{
  ColumnFamilyDescriptor, DBCompressionType, FlushOptions, LogLevel,
  MultiThreaded, OptimisticTransactionDB, Options as RocksOptions,
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
  pub fn new(path: PathBuf) -> DatabaseResult<Self> {
    Self::new_with_cache(path, None)
  }

  pub fn new_with_cache(
    path: PathBuf,
    cache: Option<Cache>,
  ) -> DatabaseResult<Self> {
    let mut opts = RocksOptions::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);
    opts.set_log_level(LogLevel::Warn);
    opts.set_db_log_dir(path.join("logs"));
    opts.set_wal_dir(path.join("wal"));
    // Keep WAL for 7 days
    opts.set_wal_ttl_seconds(60 * 60 * 24 * 7);
    // WAL size limit
    opts.set_wal_size_limit_mb(50);
    // TODO: set this flag to true: `track_and_verify_wals_in_manifest`
    // indexes_cf_options.set_arena_block_size(size)
    // indexes_cf_options.set_max_open_files(nfiles)
    opts.set_max_background_jobs(1);

    // this isn't neessary in WAL mode but set it anyways
    opts.set_atomic_flush(true);
    if let Some(cache) = cache {
      opts.set_row_cache(&cache);
    }

    let mut indexes_cf_options = RocksOptions::default();
    indexes_cf_options.set_enable_blob_files(false);
    indexes_cf_options.set_compression_type(DBCompressionType::Lz4);

    let mut rows_cf_options = RocksOptions::default();
    rows_cf_options.set_enable_blob_files(true);
    // TODO: set min blob size so that vector embeddings aren't stored in
    // blobs but documents are
    rows_cf_options.set_enable_blob_gc(true);
    rows_cf_options.set_compression_type(DBCompressionType::Lz4);
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
          indexes_cf_options,
        ),
        ColumnFamilyDescriptor::new(
          KeyValueGroup::Rows.to_string(),
          rows_cf_options,
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
