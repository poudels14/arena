use std::cell::UnsafeCell;
use std::sync::Arc;

use rocksdb::{BoundColumnFamily, DBRawIteratorWithThreadMode};
use rocksdb::{ReadOptions, Transaction as RocksTransaction};

use super::storage::RocksDatabase;

pub struct RawIterator<'a> {
  pub(super) iter:
    DBRawIteratorWithThreadMode<'a, RocksTransaction<'a, RocksDatabase>>,
}

impl<'a> RawIterator<'a> {
  pub(super) fn new(
    txn: &UnsafeCell<Option<RocksTransaction<'static, RocksDatabase>>>,
    cf: &Arc<BoundColumnFamily<'static>>,
    prefix: &[u8],
  ) -> Self {
    let txn = unsafe { txn.get().as_ref() }
      .as_ref()
      .unwrap()
      .as_ref()
      .unwrap();

    let mut opts = ReadOptions::default();
    // TODO: pass this as option
    opts.set_readahead_size(4 * 1024 * 1024);
    opts.set_prefix_same_as_start(true);
    // TODO: pass this as option
    opts.fill_cache(true);
    let mut iter = txn.raw_iterator_cf_opt(cf, opts);
    iter.seek(prefix);

    Self { iter }
  }
}

impl<'a> crate::storage::RawIterator for RawIterator<'a> {
  #[inline]
  fn key(&self) -> Option<&[u8]> {
    self.iter.key()
  }

  #[inline]
  fn value(&self) -> Option<&[u8]> {
    self.iter.value()
  }

  #[inline]
  fn get(&mut self) -> Option<(&[u8], &[u8])> {
    self.iter.item()
  }

  #[inline]
  fn next(&mut self) {
    self.iter.next();
  }
}
