use std::sync::Arc;

use dashmap::DashMap;
use derivative::Derivative;
use derive_builder::Builder;
use tokio::sync::{Mutex, OwnedMutexGuard, OwnedRwLockWriteGuard, RwLock};

use crate::Result;

#[derive(Builder, Clone, Debug)]
pub struct SchemaLocks {
  schema: Arc<str>,
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
      schema: self.schema.clone(),
      table: table_name.into(),
      lock: Arc::new(owned_lock),
    })
  }
}

#[derive(Clone, Debug)]
pub struct TableSchemaWriteLock {
  pub schema: Arc<str>,
  pub table: Arc<str>,
  pub lock: Arc<OwnedRwLockWriteGuard<String>>,
}

pub struct AdvisoryLock {
  id: i64,
  locks: Arc<DashMap<i64, Arc<Mutex<i64>>>>,
  active_locks: Arc<DashMap<i64, OwnedMutexGuard<i64>>>,
}

impl Drop for AdvisoryLock {
  fn drop(&mut self) {
    self.locks.remove(&self.id);
    self.active_locks.remove(&self.id);
  }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct AdvisoryLocks {
  #[derivative(Debug = "ignore")]
  locks: Arc<DashMap<i64, Arc<Mutex<i64>>>>,
  active_locks: Arc<DashMap<i64, OwnedMutexGuard<i64>>>,
}

impl AdvisoryLocks {
  pub fn new() -> Self {
    Self {
      locks: Arc::new(DashMap::new()),
      active_locks: Arc::new(DashMap::new()),
    }
  }

  #[tracing::instrument(level = "trace")]
  pub async fn acquire_lock(&self, id: i64) -> Result<AdvisoryLock> {
    let guard = if let Some(lock) = self.locks.get(&id) {
      lock.clone().lock_owned().await
    } else {
      let new_lock = Arc::new(Mutex::new(id));
      let old_lock = self.locks.insert(id, new_lock.clone());
      // If old lock isn't None because of race condition, put back the old
      // lock and wait on it
      // TODO: verify race condition
      if let Some(old_lock) = old_lock {
        self.locks.insert(id, old_lock.clone());
        old_lock.lock_owned().await
      } else {
        new_lock.lock_owned().await
      }
    };

    self.active_locks.insert(id, guard);
    Ok(AdvisoryLock {
      id,
      locks: self.locks.clone(),
      active_locks: self.active_locks.clone(),
    })
  }

  #[tracing::instrument(level = "trace")]
  pub async fn release_lock(&self, id: i64) -> Result<()> {
    self.locks.remove(&id);
    self.active_locks.remove(&id);
    Ok(())
  }
}
