use std::cell::UnsafeCell;

use rocksdb::DBRawIteratorWithThreadMode;
use rocksdb::{ReadOptions, Transaction as RocksTransaction};

use super::storage::RocksDatabase;
use crate::storage::transaction;

pub struct RawIterator<'a> {
  pub(super) iter:
    DBRawIteratorWithThreadMode<'a, RocksTransaction<'a, RocksDatabase>>,
}

impl<'a> RawIterator<'a> {
  pub(super) fn new(
    txn: &UnsafeCell<Option<RocksTransaction<'static, RocksDatabase>>>,
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
    opts.fill_cache(false);
    let mut iter = txn.raw_iterator_opt(opts);
    iter.seek(prefix);

    Self { iter }
  }
}

impl<'a> transaction::RawIterator for RawIterator<'a> {
  #[inline]
  fn key(&self) -> Option<&[u8]> {
    self.iter.key()
  }

  #[inline]
  fn value(&self) -> Option<&[u8]> {
    self.iter.value()
  }

  #[inline]
  fn get(&self) -> Option<(&[u8], &[u8])> {
    self.iter.item()
  }

  #[inline]
  fn next(&mut self) {
    self.iter.next();
  }
}
