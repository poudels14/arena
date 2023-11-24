use std::sync::{Arc, Mutex};

use derivative::Derivative;
use rocksdb::{Direction, IteratorMode, TransactionOptions, WriteOptions};
use rocksdb::{ReadOptions, Transaction as RocksTransaction};

use super::{RocksDatabase, StorageInner};
use crate::error::Error;
use crate::runtime::RuntimeEnv;
use crate::Result as DatabaseResult;

#[derive(Derivative, Clone)]
#[allow(unused)]
pub struct Transaction {
  runtime: RuntimeEnv,
  db: Arc<StorageInner>,
  txn: Arc<Mutex<Option<RocksTransaction<'static, RocksDatabase>>>>,
}

unsafe impl Send for Transaction {}

impl Transaction {
  pub(super) fn new(
    runtime: RuntimeEnv,
    db: Arc<StorageInner>,
  ) -> DatabaseResult<Self> {
    let db = db.clone();
    let mut txn_opt = TransactionOptions::default();
    txn_opt.set_lock_timeout(10_000);
    let txn = db.rocks.transaction_opt(&WriteOptions::default(), &txn_opt);
    let txn = Arc::new(Mutex::new(Some(unsafe {
      std::mem::transmute::<
        RocksTransaction<'_, RocksDatabase>,
        RocksTransaction<'static, RocksDatabase>,
      >(txn)
    })));

    Ok(Self { runtime, db, txn })
  }
}

impl crate::storage::Transaction for Transaction {
  fn atomic_update(
    &self,
    key: &[u8],
    updater: &dyn Fn(Option<Vec<u8>>) -> Vec<u8>,
  ) -> DatabaseResult<Vec<u8>> {
    let mut txn_opt = TransactionOptions::default();
    txn_opt.set_lock_timeout(10_000);
    let txn = self
      .db
      .rocks
      .transaction_opt(&WriteOptions::default(), &txn_opt);
    Ok(txn.get_for_update(key, true).and_then(|old| {
      let new_value = updater(old);
      txn
        .put(key, &new_value)
        .and_then(|_| txn.commit())
        .map(|_| new_value)
    })?)
  }

  fn get(&self, key: &[u8]) -> DatabaseResult<Option<Vec<u8>>> {
    let txn = self.txn.lock().unwrap();
    let txn = txn.as_ref();
    Ok(txn.ok_or(Error::TransactionFinished)?.get(key)?)
  }

  fn get_for_update(
    &self,
    key: &[u8],
    exclusive: bool,
  ) -> DatabaseResult<Option<Vec<u8>>> {
    let txn = self.txn.lock().unwrap();
    let txn = txn.as_ref();

    let mut opts = ReadOptions::default();
    opts.fill_cache(true);
    Ok(
      txn
        .ok_or(Error::TransactionFinished)?
        .get_for_update_opt(key, exclusive, &opts)?,
    )
  }

  fn scan(&self, prefix: &[u8]) -> DatabaseResult<Vec<(Box<[u8]>, Box<[u8]>)>> {
    let txn = self.txn.lock().unwrap();
    let txn = txn.as_ref();

    let txn = txn.ok_or(Error::TransactionFinished)?;

    let mut opts = ReadOptions::default();
    opts.set_prefix_same_as_start(true);
    // TODO: pass this as option
    opts.fill_cache(false);
    let iter = txn.iterator_opt(
      IteratorMode::From(prefix.as_ref(), Direction::Forward),
      opts,
    );
    Ok(iter.collect::<Result<Vec<(Box<[u8]>, Box<[u8]>)>, rocksdb::Error>>()?)
  }

  fn put(&self, key: &[u8], value: &[u8]) -> DatabaseResult<()> {
    self.put_all(&vec![(key, value)])
  }

  fn put_all(&self, rows: &[(&[u8], &[u8])]) -> DatabaseResult<()> {
    let txn = self.txn.lock().unwrap();
    let txn = txn.as_ref();

    let txn = txn.ok_or(Error::TransactionFinished)?;
    Ok(
      rows
        .into_iter()
        .map(|row| txn.put(row.0, row.1))
        .collect::<Result<Vec<()>, rocksdb::Error>>()
        .map(|_| ())?,
    )
  }

  fn commit(&self) -> DatabaseResult<()> {
    let mut txn = self.txn.lock().unwrap();
    let txn = std::mem::replace(&mut *txn, None);
    Ok(txn.ok_or(Error::TransactionFinished)?.commit()?)
  }

  fn rollback(&self) -> DatabaseResult<()> {
    let mut txn = self.txn.lock().unwrap();
    let txn = std::mem::replace(&mut *txn, None);
    Ok(txn.ok_or(Error::TransactionFinished)?.rollback()?)
  }

  fn done(&self) -> DatabaseResult<bool> {
    let txn = self.txn.lock().unwrap();
    let txn = txn.as_ref();
    Ok(txn.is_none())
  }
}
