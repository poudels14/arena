use std::cell::UnsafeCell;
use std::sync::Arc;

use rocksdb::{BoundColumnFamily, DBRawIteratorWithThreadMode};
use rocksdb::{ReadOptions, Transaction as RocksTransaction};

use super::storage::RocksDatabase;

pub struct PrefixIterator<'a> {
  prefix: Vec<u8>,
  iter: DBRawIteratorWithThreadMode<'a, RocksTransaction<'a, RocksDatabase>>,
  done: bool,
}

impl<'a> PrefixIterator<'a> {
  pub(super) fn new(
    txn: &UnsafeCell<Option<RocksTransaction<'static, RocksDatabase>>>,
    cf: &Arc<BoundColumnFamily<'static>>,
    prefix: Vec<u8>,
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
    iter.seek(&prefix);

    Self {
      prefix,
      iter,
      done: false,
    }
  }
}

impl<'a> crate::storage::PrefixIterator for PrefixIterator<'a> {
  #[inline]
  fn key(&self) -> Option<&[u8]> {
    if self.done {
      None
    } else {
      self.iter.key()
    }
  }

  #[inline]
  fn get(&self) -> Option<(&[u8], &[u8])> {
    if self.done {
      None
    } else {
      self.iter.item()
    }
  }

  #[inline]
  fn next(&mut self) {
    if !self.done {
      self.iter.next();
      let next_key = self.iter.key();
      // If prefix doesn't match, mark it as done
      if !next_key
        .map(|key| {
          // return true if prefix matches
          key.len() >= self.prefix.len()
            && key[0..self.prefix.len()] == *self.prefix
        })
        .unwrap_or(false)
      {
        self.done = true;
      }
    }
  }
}
