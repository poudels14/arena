use std::cell::UnsafeCell;
use std::sync::Arc;

use derivative::Derivative;
use parking_lot::{Mutex, MutexGuard};

use crate::{Error, Result};

#[derive(Derivative, Clone)]
#[allow(unused)]
pub struct Transaction {
  inner: Arc<Mutex<LockedTransaction>>,
}

unsafe impl Send for Transaction {}
unsafe impl Sync for Transaction {}

impl Transaction {
  pub fn new(kv: Box<dyn KeyValueProvider>) -> Self {
    Self {
      inner: Arc::new(Mutex::new(LockedTransaction {
        transaction: UnsafeCell::new(Some(kv)),
      })),
    }
  }

  pub fn lock<'a>(&'a self) -> MutexGuard<'a, LockedTransaction> {
    self.inner.lock()
  }

  pub fn commit(self) -> Result<()> {
    if self.inner.is_locked() {
      return Err(Error::ExecutionError(
        "Transaction can't be committed because it's currently locked"
          .to_owned(),
      ));
    }
    self.inner.lock().commit()
  }

  pub fn rollback(self) -> Result<()> {
    if self.inner.is_locked() {
      return Err(Error::ExecutionError(
        "Transaction can't be rolled back because it's currently locked"
          .to_owned(),
      ));
    }
    self.inner.lock().rollback()
  }

  pub fn done(&self) -> Result<bool> {
    Ok(self.inner.lock().txn().is_err())
  }
}

/// Uses interior mutability to store the KeyValue provider trait
/// because owned reference to the trait is required in order to
/// commit the transaction
pub struct LockedTransaction {
  transaction: UnsafeCell<Option<Box<dyn KeyValueProvider>>>,
}

impl LockedTransaction {
  pub fn atomic_update(
    &self,
    key: &[u8],
    updater: &dyn Fn(Option<Vec<u8>>) -> Vec<u8>,
  ) -> Result<Vec<u8>> {
    self.txn()?.atomic_update(key, updater)
  }

  pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
    self.txn()?.get(key).map_err(|e| e.to_owned())
  }

  pub fn get_or_log_error(&self, key: &[u8]) -> Option<Vec<u8>> {
    self.txn().and_then(|txn| txn.get(key)).unwrap_or_else(|e| {
      tracing::error!("Error loading key-value from storage: {:?}", e);
      None
    })
  }

  pub fn get_for_update(
    &self,
    key: &[u8],
    exclusive: bool,
  ) -> Result<Option<Vec<u8>>> {
    self.txn()?.get_for_update(key, exclusive)
  }

  #[inline]
  pub fn scan(&self, prefix: &[u8]) -> Result<Vec<(Box<[u8]>, Box<[u8]>)>> {
    self.txn()?.scan(prefix)
  }

  #[inline]
  pub fn scan_raw(&self, prefix: &[u8]) -> Result<Box<dyn RawIterator>> {
    self.txn()?.scan_raw(prefix)
  }

  #[inline]
  pub fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
    self.txn()?.put(key, value)
  }

  #[inline]
  pub fn put_all(&self, rows: &[(&[u8], &[u8])]) -> Result<()> {
    self.txn()?.put_all(rows)
  }

  #[inline]
  pub(super) fn commit(&mut self) -> Result<()> {
    unsafe { std::mem::replace(self.transaction.get().as_mut().unwrap(), None) }
      .ok_or(Error::TransactionFinished)?
      .commit()
  }

  #[inline]
  pub(super) fn rollback(&self) -> Result<()> {
    unsafe { std::mem::replace(self.transaction.get().as_mut().unwrap(), None) }
      .ok_or(Error::TransactionFinished)?
      .rollback()
  }

  #[inline]
  fn txn<'a>(&'a self) -> Result<&Box<dyn KeyValueProvider>> {
    unsafe { self.transaction.get().as_ref() }
      .unwrap()
      .as_ref()
      .ok_or(Error::TransactionFinished)
  }
}

/// This is the interface to write key/values to the database.
/// The implementation of this trait doesn't have to be thread
/// safe since the transaction manager ensures the thread safety.
pub trait KeyValueProvider {
  /// Update the value of the given key atomically.
  /// This should return error if the key was modified
  /// by another transaction after the value was read first by
  /// this transaction.
  fn atomic_update(
    &self,
    key: &[u8],
    updater: &dyn Fn(Option<Vec<u8>>) -> Vec<u8>,
  ) -> Result<Vec<u8>>;

  fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

  fn get_for_update(
    &self,
    key: &[u8],
    exclusive: bool,
  ) -> Result<Option<Vec<u8>>>;

  fn scan(&self, prefix: &[u8]) -> Result<Vec<(Box<[u8]>, Box<[u8]>)>>;

  fn scan_raw(&self, _prefix: &[u8]) -> Result<Box<dyn RawIterator>> {
    unimplemented!()
  }

  fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
    self.put_all(&vec![(key, value)])
  }

  fn put_all(&self, rows: &[(&[u8], &[u8])]) -> Result<()>;

  fn commit(&self) -> Result<()>;

  fn rollback(&self) -> Result<()>;
}

pub trait RawIterator {
  fn key(&self) -> Option<&[u8]>;

  fn value(&self) -> Option<&[u8]>;

  fn get(&self) -> Option<(&[u8], &[u8])>;

  fn next(&mut self);
}
