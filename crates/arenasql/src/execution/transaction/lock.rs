use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use strum_macros::FromRepr;

use crate::error::Error;
use crate::execution::factory::{SchemaFactory, StorageFactoryState};
use crate::Result;

#[derive(Clone)]
pub struct TransactionLock {
  pub(super) lock: Arc<AtomicUsize>,
  pub(super) schema_factories: Arc<BTreeMap<String, Arc<SchemaFactory>>>,
  pub(super) storage_factory_state: Arc<StorageFactoryState>,
}

#[derive(Debug, FromRepr)]
#[repr(usize)]
enum LockState {
  Unknown = 0,
  Free = 1,
  // Any number between [2 - (MAX - 1)]
  // means it's read locked
  // ReadLocked => 2 <-> (MAX - 1),
  WriteLocked = usize::MAX - 1,
  Closed = usize::MAX,
}

impl TransactionLock {
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
