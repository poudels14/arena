use std::path::PathBuf;
use std::sync::Arc;

use derivative::Derivative;
use getset::Getters;
use rocksdb::backup::{BackupEngine, BackupEngineOptions, RestoreOptions};
pub use rocksdb::Cache;
use rocksdb::{
  ColumnFamilyDescriptor, DBCompressionType, Env, FlushOptions, LogLevel,
  MultiThreaded, OptimisticTransactionDB, Options as RocksOptions,
};

use super::KeyValueStore;
use crate::storage::{self, KeyValueGroup, KeyValueStoreProvider};
use crate::Result as DatabaseResult;

pub(super) type RocksDatabase = OptimisticTransactionDB<MultiThreaded>;

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone)]
pub struct RocksStorage {
  #[getset(get = "pub")]
  db: Arc<RocksDatabase>,
}

impl RocksStorage {
  pub fn new(path: PathBuf) -> DatabaseResult<Self> {
    Self::new_with_cache(path, None)
  }

  pub fn load_from_backup(
    backup_dir: &str,
    db_dir: PathBuf,
    cache: Option<Cache>,
  ) -> DatabaseResult<Self> {
    let backup_opts = BackupEngineOptions::new(backup_dir)?;
    let mut env = Env::new()?;
    env.set_background_threads(4);
    let mut engine = BackupEngine::open(&backup_opts, &env)?;

    let restore_opts = RestoreOptions::default();
    engine.restore_from_latest_backup(
      &db_dir,
      db_dir.join("wal"),
      &restore_opts,
    )?;

    Self::new_with_cache(db_dir, cache)
  }

  pub fn new_with_cache(
    db_dir: PathBuf,
    cache: Option<Cache>,
  ) -> DatabaseResult<Self> {
    let mut opts = RocksOptions::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);
    opts.set_log_level(LogLevel::Warn);
    opts.set_db_log_dir(db_dir.join("logs"));
    opts.set_wal_dir(db_dir.join("wal"));
    // Keep WAL for 7 days
    opts.set_wal_ttl_seconds(60 * 60 * 24 * 7);
    // WAL size limit
    opts.set_wal_size_limit_mb(50);
    // TODO: set this flag to true: `track_and_verify_wals_in_manifest`
    // indexes_cf_options.set_arena_block_size(size)
    // indexes_cf_options.set_max_open_files(nfiles)
    opts.set_max_background_jobs(1);
    // Dump stats every 1 min for now
    opts.set_stats_dump_period_sec(60);

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
    let db: RocksDatabase = OptimisticTransactionDB::open_cf_descriptors(
      &opts,
      db_dir,
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
          KeyValueGroup::IndexRows.to_string(),
          indexes_cf_options,
        ),
        ColumnFamilyDescriptor::new(
          KeyValueGroup::Rows.to_string(),
          rows_cf_options,
        ),
      ],
    )?;
    Ok(Self { db: Arc::new(db) })
  }

  pub fn get_db_size(&self) -> DatabaseResult<usize> {
    let live_files = self.db.live_files()?;
    let total_size = live_files.iter().map(|f| f.size).sum();
    Ok(total_size)
  }

  pub fn compact_and_flush(&self) -> DatabaseResult<()> {
    let db = &self.db;
    db.compact_range(None::<&[u8]>, None::<&[u8]>);

    let mut flush_opt = FlushOptions::default();
    flush_opt.set_wait(true);
    db.flush()?;
    Ok(())
  }
}

impl Drop for RocksStorage {
  fn drop(&mut self) {
    // Note: wait for the flush to complete after the storage is dropped
    let mut opts = FlushOptions::default();
    opts.set_wait(true);
    self.db.flush_opt(&opts).unwrap();
  }
}

impl KeyValueStoreProvider for RocksStorage {
  fn as_any(&self) -> &dyn std::any::Any {
    self
  }

  fn new_transaction(&self) -> DatabaseResult<Box<dyn storage::KeyValueStore>> {
    Ok(Box::new(KeyValueStore::new(self.db.clone())?))
  }
}
