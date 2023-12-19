use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use derivative::Derivative;
use derive_builder::Builder;
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
  #[builder(default = "Arc::new(BTreeMap::new())")]
  schema_factories: Arc<BTreeMap<String, Arc<SchemaFactory>>>,
  storage_factory_state: StorageFactoryState,
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
    schema: &str,
    table_name: &str,
  ) -> Result<TableSchemaWriteLock> {
    self
      .schema_factories
      .get(schema)
      .unwrap()
      .acquire_table_schema_write_lock(table_name)
      .await
  }

  /// Holds the write lock to the table until this transaction is dropped
  pub fn hold_table_schema_lock(
    &self,
    lock: TableSchemaWriteLock,
  ) -> Result<()> {
    self
      .schema_factories
      .get(lock.schema.as_ref())
      .unwrap()
      .hold_table_schema_lock(lock)
  }

  pub fn get_table(&self, schema: &str, name: &str) -> Option<Arc<Table>> {
    self
      .schema_factories
      .get(schema)
      .and_then(|sf| sf.get_table(name))
  }

  pub fn table_names(&self, schema: &str) -> Vec<String> {
    self
      .schema_factories
      .get(schema)
      .map(|sf| sf.table_names())
      .unwrap_or_default()
  }

  #[inline]
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

  #[inline]
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

  #[inline]
  pub fn closed(&self) -> bool {
    match LockState::from_repr(self.lock.load(Ordering::SeqCst)) {
      Some(LockState::Closed) => true,
      _ => false,
    }
  }

  #[inline]
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

    if self
      .schema_factories
      .values()
      .map(|t| t.locked_tables.lock().len())
      .sum::<usize>()
      > 0
    {
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
