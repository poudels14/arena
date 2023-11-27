use rocksdb::{
  DBCompressionType, MultiThreaded, OptimisticTransactionDB, Options,
};

use crate::Result;

pub(super) type RocksDatabase = OptimisticTransactionDB<MultiThreaded>;

pub fn open(path: &str) -> Result<RocksDatabase> {
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

  let rocks: RocksDatabase = OptimisticTransactionDB::open(&opts, path)?;
  Ok(rocks)
}
