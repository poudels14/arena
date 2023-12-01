use std::cell::UnsafeCell;
use std::sync::Arc;

use rocksdb::{
  Direction, IteratorMode, OptimisticTransactionOptions, WriteOptions,
};
use rocksdb::{ReadOptions, Transaction as RocksTransaction};

use super::iterator::RawIterator as RocksRawIterator;
use super::storage::RocksDatabase;
use crate::storage::transaction::RawIterator;
use crate::Result as DatabaseResult;

/// The rocks db transaction is stored in UnsafeCell for interior
/// mutability since transaction.commit() requires owned object.
/// It's okay to use unsafe cell here since the transaction manager
/// ensures thread safety
pub struct KeyValueProvider<'a> {
  kv: Arc<RocksDatabase>,
  txn: UnsafeCell<Option<RocksTransaction<'a, RocksDatabase>>>,
}

impl<'a> KeyValueProvider<'a> {
  pub(super) fn new(kv: Arc<RocksDatabase>) -> DatabaseResult<Self> {
    let db = kv.clone();
    let mut txn_opt = OptimisticTransactionOptions::default();
    txn_opt.set_snapshot(true);
    let txn = db.transaction_opt(&WriteOptions::default(), &txn_opt);

    let txn = unsafe {
      std::mem::transmute::<
        RocksTransaction<'_, RocksDatabase>,
        RocksTransaction<'static, RocksDatabase>,
      >(txn)
    };

    Ok(Self {
      kv,
      txn: UnsafeCell::new(Some(txn)),
    })
  }

  #[inline]
  fn txn(&self) -> &RocksTransaction<'a, RocksDatabase> {
    unsafe { self.txn.get().as_ref() }
      .unwrap()
      .as_ref()
      .unwrap()
  }
}

impl<'a> crate::storage::KeyValueProvider for KeyValueProvider<'a> {
  fn atomic_update(
    &self,
    key: &[u8],
    updater: &dyn Fn(Option<Vec<u8>>) -> Vec<u8>,
  ) -> DatabaseResult<Vec<u8>> {
    let mut txn_opt = OptimisticTransactionOptions::default();
    txn_opt.set_snapshot(true);
    let txn = self.kv.transaction_opt(&WriteOptions::default(), &txn_opt);
    Ok(txn.get_for_update(key, true).and_then(|old| {
      let new_value = updater(old);
      txn
        .put(key, &new_value)
        .and_then(|_| txn.commit())
        .map(|_| new_value)
    })?)
  }

  fn get(&self, key: &[u8]) -> DatabaseResult<Option<Vec<u8>>> {
    Ok(self.txn().get(key)?)
  }

  fn get_for_update(
    &self,
    key: &[u8],
    exclusive: bool,
  ) -> DatabaseResult<Option<Vec<u8>>> {
    let mut opts = ReadOptions::default();
    opts.fill_cache(true);
    Ok(self.txn().get_for_update_opt(key, exclusive, &opts)?)
  }

  fn scan(&self, prefix: &[u8]) -> DatabaseResult<Vec<(Box<[u8]>, Box<[u8]>)>> {
    let mut opts = ReadOptions::default();
    opts.set_readahead_size(4 * 1024 * 1024);
    opts.set_prefix_same_as_start(true);
    // TODO: pass this as option
    opts.fill_cache(true);
    let iter = self.txn().iterator_opt(
      IteratorMode::From(prefix.as_ref(), Direction::Forward),
      opts,
    );
    Ok(iter.collect::<Result<Vec<(Box<[u8]>, Box<[u8]>)>, rocksdb::Error>>()?)
  }

  fn scan_raw(&self, prefix: &[u8]) -> DatabaseResult<Box<dyn RawIterator>> {
    let txn = unsafe {
      std::mem::transmute::<
        &UnsafeCell<Option<RocksTransaction<'a, RocksDatabase>>>,
        &UnsafeCell<Option<RocksTransaction<'static, RocksDatabase>>>,
      >(&self.txn)
    };

    Ok(Box::new(RocksRawIterator::new(&txn, prefix)))
  }

  fn put(&self, key: &[u8], value: &[u8]) -> DatabaseResult<()> {
    self.put_all(&vec![(key, value)])
  }

  fn put_all(&self, rows: &[(&[u8], &[u8])]) -> DatabaseResult<()> {
    let txn = self.txn();
    Ok(
      rows
        .into_iter()
        .map(|row| txn.put(row.0, row.1))
        .collect::<Result<Vec<()>, rocksdb::Error>>()
        .map(|_| ())?,
    )
  }

  /// Once a rocksdb transaction is committed, it shouldn't be used
  /// again. If used again, it will panic
  fn commit(&self) -> DatabaseResult<()> {
    let t =
      unsafe { std::mem::replace(self.txn.get().as_mut().unwrap(), None) }
        .unwrap();
    Ok(t.commit()?)
  }

  /// Once a rocksdb transaction is rolled back, it shouldn't be used
  /// again. If used again, it will panic
  fn rollback(&self) -> DatabaseResult<()> {
    Ok(self.txn().rollback()?)
  }
}
