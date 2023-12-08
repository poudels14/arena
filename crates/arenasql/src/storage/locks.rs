use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use dashmap::DashMap;
use derive_builder::Builder;
use strum_macros::FromRepr;
use tokio::sync::{OwnedRwLockWriteGuard, RwLock};

use crate::schema::Table;
use crate::Result;

#[derive(Builder, Clone, Debug)]
pub struct SchemaLocks {
  schema_reload_flag: Arc<AtomicBool>,

  #[builder(setter(skip), default = "Arc::new(DashMap::new())")]
  table_locks: Arc<DashMap<String, Arc<RwLock<String>>>>,
}

impl SchemaLocks {
  pub async fn acquire_table_schema_write_lock(
    &self,
    table_name: &str,
  ) -> Result<TableSchemaWriteLock> {
    let owned_lock = match self.table_locks.get(table_name) {
      Some(existin_lock) => existin_lock.clone().write_owned().await,
      None => {
        let lock = Arc::new(RwLock::new(table_name.to_owned()));
        self.table_locks.insert(table_name.to_owned(), lock.clone());
        lock.write_owned().await
      }
    };

    Ok(TableSchemaWriteLock {
      table: None,
      lock: Arc::new(owned_lock),
      schema_reload_flag: self.schema_reload_flag.clone(),
    })
  }
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

impl TransactionLock {
  pub fn new(lock: Option<Arc<AtomicUsize>>) -> Self {
    Self { lock }
  }
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

#[derive(Clone)]
pub struct TableSchemaWriteLock {
  pub lock: Arc<OwnedRwLockWriteGuard<String>>,
  pub table: Option<Arc<Table>>,
  pub schema_reload_flag: Arc<AtomicBool>,
}

impl Drop for TableSchemaWriteLock {
  fn drop(&mut self) {
    self.schema_reload_flag.store(true, Ordering::Release);
  }
}
