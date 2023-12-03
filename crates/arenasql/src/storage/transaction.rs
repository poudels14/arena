use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use derivative::Derivative;
use strum_macros::FromRepr;

use super::kvprovider::KeyValueProvider;
use super::operators::StorageOperator;
use super::Serializer;
use crate::{Error, Result};

#[derive(Derivative, Clone)]
#[allow(unused)]
pub struct Transaction {
  kv: Arc<Box<dyn KeyValueProvider>>,
  serializer: Serializer,
  state: Arc<AtomicUsize>,
}

#[derive(Debug, FromRepr)]
#[repr(usize)]
pub(super) enum TransactionState {
  Unknown = 0,
  Free = 1,
  Locked = 2,
  Closed = 3,
}

unsafe impl Send for Transaction {}
unsafe impl Sync for Transaction {}

impl Transaction {
  pub fn new(kv: Box<dyn KeyValueProvider>, serializer: Serializer) -> Self {
    let state = Arc::new(AtomicUsize::new(TransactionState::Free as usize));
    Self {
      kv: Arc::new(kv),
      serializer,
      state,
    }
  }

  pub fn lock<'a>(&'a self) -> Result<StorageOperator> {
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

    Ok(StorageOperator {
      kv: self.kv.clone(),
      serializer: self.serializer.clone(),
      lock: self.state.clone(),
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
