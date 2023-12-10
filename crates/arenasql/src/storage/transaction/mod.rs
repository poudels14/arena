mod state;

use std::sync::Arc;

use derivative::Derivative;
use derive_builder::Builder;

use super::handler::StorageHandler;
use super::kvstore::KeyValueStore;
use super::Serializer;
use crate::Result;

pub use state::{TransactionState, TransactionStateBuilder};

#[derive(Builder, Derivative, Clone)]
#[allow(unused)]
pub struct Transaction {
  pub serializer: Serializer,
  kv_store: Arc<Box<dyn KeyValueStore>>,
  state: Arc<TransactionState>,
}

unsafe impl Send for Transaction {}
unsafe impl Sync for Transaction {}

impl Transaction {
  pub fn state(&self) -> &TransactionState {
    &self.state.as_ref()
  }

  // TODO: return mutexlock or some type that is not Send+Sync
  // and gets dropped when it's out of scope so that deadlock error
  // is easily prevented
  // TODO: change this to read/write lock since SELECT that uses more
  // than one table will need more than 1 lock at once
  pub fn lock<'a>(&'a self, exclusive: bool) -> Result<StorageHandler> {
    self.state.lock(exclusive)?;
    Ok(StorageHandler {
      kv: self.kv_store.clone(),
      serializer: self.serializer.clone(),
      transaction_state: Some(self.state.clone()),
    })
  }

  pub fn commit(&self) -> Result<()> {
    self.state.close()?;
    self.kv_store.commit()?;
    Ok(())
  }

  pub fn rollback(self) -> Result<()> {
    self.state.close()?;
    self.kv_store.rollback()?;
    Ok(())
  }
}
