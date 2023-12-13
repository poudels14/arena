use std::cell::UnsafeCell;
use std::sync::Arc;

use rocksdb::{BoundColumnFamily, OptimisticTransactionOptions, WriteOptions};
use rocksdb::{ReadOptions, Transaction as RocksTransaction};
use strum::IntoEnumIterator;

use super::iterator::PrefixIterator as RocksRawIterator;
use super::storage::RocksDatabase;
use crate::storage::{KeyValueGroup, KeyValueIterator};
use crate::{Error, Result as DatabaseResult};

/// The rocks db transaction is stored in UnsafeCell for interior
/// mutability since transaction.commit() requires owned object.
/// It's okay to use unsafe cell here since the transaction manager
/// ensures thread safety
pub struct KeyValueStore {
  kv: Arc<RocksDatabase>,
  transaction: UnsafeCell<Option<RocksTransaction<'static, RocksDatabase>>>,
  cfs: Vec<Arc<BoundColumnFamily<'static>>>,
}

impl KeyValueStore {
  pub(super) fn new(kv: Arc<RocksDatabase>) -> DatabaseResult<Self> {
    let db = kv.clone();
    let mut txn_opt = OptimisticTransactionOptions::default();
    txn_opt.set_snapshot(true);
    let txn = db.transaction_opt(&WriteOptions::default(), &txn_opt);

    let transaction = unsafe {
      std::mem::transmute::<
        RocksTransaction<'_, RocksDatabase>,
        RocksTransaction<'static, RocksDatabase>,
      >(txn)
    };

    let cfs = KeyValueGroup::iter()
      .map(|group| {
        db.cf_handle(&group.to_string())
          .ok_or(Error::IOError(format!(
            "Failed to get ColumnFamily handle: {}",
            group.to_string()
          )))
          .map(|handle| unsafe {
            std::mem::transmute::<
              Arc<BoundColumnFamily<'_>>,
              Arc<BoundColumnFamily<'static>>,
            >(handle)
          })
      })
      .collect::<DatabaseResult<Vec<Arc<BoundColumnFamily<'static>>>>>()?;

    Ok(Self {
      kv,
      transaction: UnsafeCell::new(Some(transaction)),
      cfs,
    })
  }

  #[inline]
  fn get_txn(&self) -> &RocksTransaction<'static, RocksDatabase> {
    unsafe { self.transaction.get().as_ref() }
      .unwrap()
      .as_ref()
      .unwrap()
  }
}

impl crate::storage::KeyValueStore for KeyValueStore {
  /// Updates the value of the given key atomically and returns the new value
  fn atomic_update(
    &self,
    group: KeyValueGroup,
    key: &[u8],
    updater: &dyn Fn(Option<Vec<u8>>) -> DatabaseResult<Vec<u8>>,
  ) -> DatabaseResult<Vec<u8>> {
    let mut txn_opt = OptimisticTransactionOptions::default();
    txn_opt.set_snapshot(true);
    let txn = self.kv.transaction_opt(&WriteOptions::default(), &txn_opt);

    let cf = &self.cfs[group as usize];
    let old_value =
      txn.get_for_update_cf_opt(cf, key, true, &Default::default())?;
    let new_value = updater(old_value)?;
    txn.put_cf(cf, key, &new_value).and_then(|_| txn.commit())?;
    Ok(new_value)
  }

  fn get(
    &self,
    group: KeyValueGroup,
    key: &[u8],
  ) -> DatabaseResult<Option<Vec<u8>>> {
    Ok(self.get_txn().get_cf(&self.cfs[group as usize], key)?)
  }

  fn get_for_update(
    &self,
    group: KeyValueGroup,
    key: &[u8],
    exclusive: bool,
  ) -> DatabaseResult<Option<Vec<u8>>> {
    let mut opts = ReadOptions::default();
    opts.fill_cache(true);
    Ok(self.get_txn().get_for_update_cf_opt(
      &self.cfs[group as usize],
      key,
      exclusive,
      &opts,
    )?)
  }

  fn scan_with_prefix(
    &self,
    group: KeyValueGroup,
    prefix: &[u8],
  ) -> DatabaseResult<Box<dyn KeyValueIterator>> {
    Ok(Box::new(RocksRawIterator::new(
      &self.transaction,
      &self.cfs[group as usize],
      prefix.to_vec(),
    )))
  }

  fn put(
    &self,
    group: KeyValueGroup,
    key: &[u8],
    value: &[u8],
  ) -> DatabaseResult<()> {
    self.put_all(group, &vec![(key, value)])
  }

  fn put_all(
    &self,
    group: KeyValueGroup,
    rows: &[(&[u8], &[u8])],
  ) -> DatabaseResult<()> {
    let txn = self.get_txn();
    let group_cf = &self.cfs[group as usize];
    Ok(
      rows
        .into_iter()
        .map(|row| txn.put_cf(group_cf, row.0, row.1))
        .collect::<Result<Vec<()>, rocksdb::Error>>()
        .map(|_| ())?,
    )
  }

  /// Once a rocksdb transaction is committed, it shouldn't be used
  /// again. If used again, it will panic
  fn commit(&self) -> DatabaseResult<()> {
    let t = unsafe {
      std::mem::replace(self.transaction.get().as_mut().unwrap(), None)
    }
    .unwrap();
    Ok(t.commit()?)
  }

  /// Once a rocksdb transaction is rolled back, it shouldn't be used
  /// again. If used again, it will panic
  fn rollback(&self) -> DatabaseResult<()> {
    Ok(self.get_txn().rollback()?)
  }
}
