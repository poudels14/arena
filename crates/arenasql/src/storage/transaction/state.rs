use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use derivative::Derivative;
use derive_builder::Builder;
use parking_lot::Mutex;
use strum_macros::FromRepr;

use crate::schema::Table;
use crate::storage::factory::{SchemaFactory, StorageFactoryState};
use crate::storage::locks::TableSchemaWriteLock;
use crate::{Error, Result};

/// Don't implement clone so that when this is dropped,
/// we can guarantee that the transaction with this state
/// was closed (committed/rolledback)
#[derive(Builder, Derivative)]
pub struct TransactionState {
  schema_factory: Arc<SchemaFactory>,
  storage_factory_state: StorageFactoryState,
  #[builder(setter(skip), default = "Arc::new(Mutex::new(vec![]))")]
  locked_tables: Arc<Mutex<Vec<TableSchemaWriteLock>>>,
  /// LockState value
  #[builder(setter(skip), default = "Arc::new(AtomicUsize::new(1))")]
  lock: Arc<AtomicUsize>,
}

impl Drop for TransactionState {
  fn drop(&mut self) {
    self.storage_factory_state.reduce_active_transaction_count();
  }
}

impl TransactionState {
  #[inline]
  pub async fn acquire_table_schema_write_lock(
    &self,
    table_name: &str,
  ) -> Result<TableSchemaWriteLock> {
    // If the transaction has an existing lock for the table, return it
    // else, acquire it from lock factory
    if let Some(lock) = self
      .locked_tables
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

  /// Holds the write lock to the table until this transaction is dropped
  pub fn hold_table_schema_lock(
    &self,
    lock: TableSchemaWriteLock,
  ) -> Result<()> {
    let mut locked_tables = self.locked_tables.lock();
    let existing_index = locked_tables.iter().position(|l| {
      l.table
        .as_ref()
        .map(|t| *t.name == *lock.lock.deref().deref())
        .unwrap_or(false)
    });
    // Remove the table from the locked tables if it exist
    // so that the list will have updated data
    if let Some(index) = existing_index {
      locked_tables.remove(index);
    }
    locked_tables.push(lock);
    Ok(())
  }

  #[inline]
  pub fn catalog(&self) -> &String {
    &self.schema_factory.catalog
  }

  #[inline]
  pub fn schema(&self) -> &String {
    &self.schema_factory.schema
  }

  pub fn get_table(&self, name: &str) -> Option<Arc<Table>> {
    // Note: need to check locked_tables first to check if the
    // table was updated by the current transaction but the change
    // hasn't been committed
    self
      .locked_tables
      .lock()
      .iter()
      .find_map(|t| {
        t.table
          .as_ref()
          .filter(|t| t.name == name)
          .map(|t| t.clone())
      })
      .or_else(|| self.schema_factory.get_table(name))
  }

  pub fn table_names(&self) -> Vec<String> {
    let mut tables = vec![
      self.schema_factory.table_names(),
      self
        .locked_tables
        .lock()
        .iter()
        .filter_map(|t| t.table.as_ref().map(|t| t.name.clone()))
        .collect(),
    ]
    .concat();
    tables.dedup();
    tables
  }

  pub fn lock(&self, exclusive: bool) -> Result<()> {
    let state = self.lock.load(Ordering::SeqCst);
    match LockState::from_repr(state) {
      Some(LockState::Closed) => {
        return Err(Error::InvalidTransactionState(
          "Transaction already closed".to_owned(),
        ));
      }
      Some(LockState::Free) => {}
      Some(LockState::WriteLocked) => {
        return Err(Error::InvalidTransactionState(
          "Failed to acquire transaction [active write lock]".to_owned(),
        ));
      }
      Some(LockState::Unknown) => {
        return Err(Error::InvalidTransactionState(format!(
          "Invalid transaction state: {:?}",
          state
        )))
      }
      // Throw error if exclusive/write lock is asked when the transaction
      // is already has read locks. This prevents reading and writing at the
      // same time
      // Anything above 2 < (MAX-1) is read locked
      None => {
        if exclusive {
          return Err(Error::InvalidTransactionState(
            "Can't acquire exclusive lock when there are active read locks"
              .to_owned(),
          ));
        }
      }
    }

    match exclusive {
      true => self
        .lock
        .compare_exchange(
          LockState::Free as usize,
          LockState::WriteLocked as usize,
          Ordering::Acquire,
          Ordering::Relaxed,
        )
        .map(|_| ())
        .map_err(|_| {
          Error::IOError("Failed to acquire transaction lock".to_owned())
        }),
      false => {
        self.lock.fetch_add(1, Ordering::AcqRel);
        Ok(())
      }
    }
  }

  pub fn unlock(&self) -> Result<()> {
    let state = self.lock.load(Ordering::SeqCst);
    match LockState::from_repr(state) {
      Some(LockState::WriteLocked) => {
        let _ = self
          .lock
          .compare_exchange(
            LockState::WriteLocked as usize,
            LockState::Free as usize,
            Ordering::Acquire,
            Ordering::Relaxed,
          )
          .unwrap();
      }
      // If read locked, reduce locks count
      // Anything above 2 < (MAX-1) is read locked
      None => {
        self.lock.fetch_sub(1, Ordering::AcqRel);
      }
      _ => {
        return Err(Error::InvalidTransactionState(format!(
          "Invalid transaction state: {:?}",
          state
        )))
      }
    }
    Ok(())
  }

  pub fn close(&self) -> Result<()> {
    match LockState::from_repr(self.lock.load(Ordering::SeqCst)) {
      Some(LockState::Free) => {}
      Some(LockState::Closed) => {
        return Err(Error::InvalidTransactionState(
          "Transaction already closed".to_owned(),
        ));
      }
      _ => {
        return Err(Error::InvalidTransactionState(
          "Cannot close a locked transaction".to_owned(),
        ));
      }
    }

    let state = self
      .lock
      .compare_exchange(
        LockState::Free as usize,
        LockState::Closed as usize,
        Ordering::Acquire,
        Ordering::Relaxed,
      )
      .unwrap_or(LockState::Unknown as usize);

    if self.locked_tables.lock().len() > 0 {
      self.storage_factory_state.reload_schema();
    }
    if state != LockState::Free as usize {
      return Err(Error::IOError("Failed to close transaction".to_owned()));
    }
    Ok(())
  }
}

#[derive(Debug, FromRepr)]
#[repr(usize)]
pub(super) enum LockState {
  Unknown = 0,
  Free = 1,
  // Any number between [2 - (MAX - 1)]
  // means it's read locked
  // ReadLocked => 2 <-> (MAX - 1),
  WriteLocked = usize::MAX - 1,
  Closed = usize::MAX,
}
