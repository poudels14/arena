use std::sync::Arc;

use dashmap::DashMap;
use derive_builder::Builder;
use tokio::sync::{OwnedRwLockWriteGuard, RwLock};

use crate::schema::Table;
use crate::Result;

#[derive(Builder, Clone, Debug)]
pub struct SchemaLocks {
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
    })
  }
}

#[derive(Clone, Debug)]
pub struct TableSchemaWriteLock {
  pub lock: Arc<OwnedRwLockWriteGuard<String>>,
  pub table: Option<Arc<Table>>,
}
