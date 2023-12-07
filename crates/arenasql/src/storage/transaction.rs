use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use derivative::Derivative;
use parking_lot::Mutex;
use strum_macros::FromRepr;

use super::handler::StorageHandler;
use super::kvstore::KeyValueStore;
use super::schema::SchemaLock;
use super::{SchemaFactory, Serializer};
use crate::{Error, Result};

#[derive(Derivative, Clone)]
#[allow(unused)]
pub struct Transaction {
  pub(crate) schema_factory: Arc<SchemaFactory>,
  pub serializer: Serializer,
  kv: Arc<Box<dyn KeyValueStore>>,
  state: Arc<AtomicUsize>,
  /// Hold all the locks acquired by this transaction
  /// until the transaction is dropped (committed/rolled back)
  /// Using parking_lot Mutex here since it's very unlikely to
  /// be contended
  locks: Arc<Mutex<Vec<SchemaLock>>>,
}

#[derive(Debug, FromRepr)]
#[repr(usize)]
pub(super) enum TransactionState {
  Unknown = 0,
  Free = 1,
  Locked = 2,
  Closed = 3,
}

pub struct TransactionLock {
  lock: Option<Arc<AtomicUsize>>,
}

impl Default for TransactionLock {
  fn default() -> Self {
    Self { lock: None }
  }
}

impl Drop for TransactionLock {
  fn drop(&mut self) {
    if let Some(lock) = self.lock.take() {
      let _ = lock.compare_exchange(
        TransactionState::Locked as usize,
        TransactionState::Free as usize,
        Ordering::Acquire,
        Ordering::Relaxed,
      );
    }
  }
}

unsafe impl Send for Transaction {}
unsafe impl Sync for Transaction {}

impl Transaction {
  pub(super) fn new(
    schema_factory: Arc<SchemaFactory>,
    kv: Arc<Box<dyn KeyValueStore>>,
    serializer: Serializer,
  ) -> Self {
    Self {
      schema_factory,
      kv,
      serializer,
      state: Arc::new(AtomicUsize::new(TransactionState::Free as usize)),
      locks: Arc::new(Mutex::new(vec![])),
    }
  }

  pub fn acquire_table_lock(&self, table: &str) -> Result<()> {
    let lock = self.schema_factory.lock_table_for_write(&table)?;
    self.locks.lock().push(lock);
    Ok(())
  }

  // TODO: return mutexlock or some type that is not Send+Sync
  // and gets dropped when it's out of scope so that deadlock error
  // is easily prevented
  // TODO: change this to read/write lock since SELECT that uses more
  // than one table will need more than 1 lock at once
  pub fn lock<'a>(&'a self) -> Result<StorageHandler> {
    match TransactionState::from_repr(self.state.load(Ordering::SeqCst)) {
      Some(TransactionState::Locked) => {
        return Err(Error::InvalidTransactionState(
          "Failed to acquire transaction [aready locked]".to_owned(),
        ));
      }
      Some(TransactionState::Closed) => {
        return Err(Error::InvalidTransactionState(
          "Transaction already closed".to_owned(),
        ));
      }
      Some(TransactionState::Free) => {}
      s => {
        return Err(Error::InvalidTransactionState(format!(
          "Invalid transaction state: {:?}",
          s
        )))
      }
    }

    let _ = self
      .state
      .compare_exchange(
        TransactionState::Free as usize,
        TransactionState::Locked as usize,
        Ordering::Acquire,
        Ordering::Relaxed,
      )
      .map_err(|_| {
        Error::IOError("Failed to acquire transaction lock".to_owned())
      })?;

    Ok(StorageHandler {
      kv: self.kv.clone(),
      serializer: self.serializer.clone(),
      lock: TransactionLock {
        lock: Some(self.state.clone()),
      },
    })
  }

  pub fn commit(&self) -> Result<()> {
    self.close_transaction()?;
    self.kv.commit()
  }

  pub fn rollback(self) -> Result<()> {
    self.close_transaction()?;
    self.kv.rollback()
  }

  /// transaction should be free when calling this
  fn close_transaction(&self) -> Result<()> {
    match TransactionState::from_repr(self.state.load(Ordering::SeqCst)) {
      Some(TransactionState::Closed) => {
        return Err(Error::InvalidTransactionState(
          "Transaction already closed".to_owned(),
        ));
      }
      Some(TransactionState::Locked) => {
        return Err(Error::InvalidTransactionState(
          "Cannot close a locked transaction".to_owned(),
        ));
      }
      _ => {}
    }

    let state = self
      .state
      .compare_exchange(
        TransactionState::Free as usize,
        TransactionState::Closed as usize,
        Ordering::Acquire,
        Ordering::Relaxed,
      )
      .unwrap_or(TransactionState::Unknown as usize);
    if state != TransactionState::Free as usize {
      return Err(Error::IOError("Failed to close transaction".to_owned()));
    }
    Ok(())
  }
}
