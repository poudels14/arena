mod transaction;
pub use transaction::Transaction;

use std::sync::Arc;

use dashmap::DashMap;
use derivative::Derivative;
use once_cell::sync::Lazy;
use rocksdb::{
  DBCompressionType, MultiThreaded, Options, TransactionDB,
  TransactionDBOptions,
};

use crate::runtime::RuntimeEnv;
use crate::Result as DatabaseResult;

pub(super) type RocksDatabase = TransactionDB<MultiThreaded>;

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct RocksStorage {
  inner: Arc<StorageInner>,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub(super) struct StorageInner {
  runtime: RuntimeEnv,
  path: String,
  #[derivative(Debug = "ignore")]
  rocks: RocksDatabase,
}

pub(super) static STORAGES: Lazy<DashMap<String, RocksStorage>> =
  Lazy::new(|| DashMap::new());

impl RocksStorage {
  pub fn open(path: &str, runtime: RuntimeEnv) -> DatabaseResult<Self> {
    if let Some(db) = STORAGES.get(path) {
      return Ok(db.clone());
    }

    let mut opts = Options::default();
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

    let rocks: RocksDatabase =
      TransactionDB::open(&opts, &TransactionDBOptions::default(), path)?;

    let handle = Self {
      inner: Arc::new(StorageInner {
        runtime,
        path: path.to_string(),
        rocks,
      }),
    };
    STORAGES.insert(path.to_string(), handle.clone());

    Ok(handle)
  }

  pub fn begin_transaction(
    &self,
  ) -> DatabaseResult<Arc<dyn super::Transaction>> {
    Ok(Arc::new(Transaction::new(
      self.inner.runtime.clone(),
      self.inner.clone(),
    )?))
  }
}

impl Drop for RocksStorage {
  fn drop(&mut self) {
    // TODO: use LRU to drop storage ref from STORAGES to ensure
    // not a lot of dbs are open at a time
    // let count = Arc::strong_count(&self.inner);
  }
}
