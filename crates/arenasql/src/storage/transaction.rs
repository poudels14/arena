use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use derivative::Derivative;
use derive_builder::Builder;
use parking_lot::Mutex;

use super::handler::StorageHandler;
use super::kvstore::KeyValueStore;
use super::locks::{TableSchemaWriteLock, TransactionLock, TransactionState};
use super::{SchemaFactory, Serializer};
use crate::schema::Table;
use crate::{Error, Result};

#[derive(Builder, Derivative, Clone)]
#[allow(unused)]
pub struct Transaction {
  pub(crate) schema_factory: Arc<SchemaFactory>,
  pub serializer: Serializer,
  kv_store: Arc<Box<dyn KeyValueStore>>,
  #[builder(
    setter(skip),
    default = "Arc::new(AtomicUsize::new(TransactionState::Free as usize))"
  )]
  state: Arc<AtomicUsize>,
  /// Hold all the locks acquired by this transaction
  /// until the transaction is dropped (committed/rolled back)
  /// Using parking_lot Mutex here since it's very unlikely to
  /// be contended
  #[builder(setter(skip), default = "Arc::new(Mutex::new(vec![]))")]
  acquired_locks: Arc<Mutex<Vec<TableSchemaWriteLock>>>,
}

unsafe impl Send for Transaction {}
unsafe impl Sync for Transaction {}

impl Transaction {
  #[inline]
  pub async fn acquire_table_schema_write_lock(
    &self,
    table_name: &str,
  ) -> Result<TableSchemaWriteLock> {
    // If the transaction has an existing lock for the table, return it
    // else, acquire it from lock factory
    if let Some(lock) = self
      .acquired_locks
      .lock()
      .iter()
      .find(|t| t.lock.deref().deref() == table_name)
    {
      return Ok(lock.clone());
    }
    self
      .schema_factory
      .schema_locks
      .acquire_table_schema_write_lock(table_name)
      .await
  }

  // Holds the write lock to the table until this transaction is dropped
  pub fn hold_table_write_lock(
    &self,
    lock: TableSchemaWriteLock,
  ) -> Result<()> {
    self.acquired_locks.lock().push(lock);
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
      kv: self.kv_store.clone(),
      serializer: self.serializer.clone(),
      lock: TransactionLock::new(Some(self.state.clone())),
    })
  }

  pub fn get_table(&self, name: &str) -> Option<Arc<Table>> {
    self.schema_factory.get_table(name).or_else(|| {
      self.acquired_locks.lock().iter().find_map(|t| {
        t.table
          .as_ref()
          .filter(|t| t.name == name)
          .map(|t| t.clone())
      })
    })
  }

  pub fn table_names(&self) -> Vec<String> {
    vec![
      self.schema_factory.table_names(),
      self
        .acquired_locks
        .lock()
        .iter()
        .filter_map(|t| t.table.as_ref().map(|t| t.name.clone()))
        .collect(),
    ]
    .concat()
  }

  pub fn commit(&self) -> Result<()> {
    self.close_transaction()?;
    self.kv_store.commit()
  }

  pub fn rollback(self) -> Result<()> {
    self.close_transaction()?;
    self.kv_store.rollback()
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
