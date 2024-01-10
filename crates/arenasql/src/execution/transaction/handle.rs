use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use derivative::Derivative;
use getset::{Getters, Setters};
use parking_lot::Mutex;
use sqlparser::ast::Statement;

use super::lock::TransactionLock;
use crate::execution::factory::{SchemaFactory, StorageFactoryState};
use crate::execution::TableSchemaWriteLock;
use crate::schema::Table;
use crate::storage::{KeyValueStore, Serializer, StorageHandler};
use crate::Result;

/// Don't implement clone so that when this is dropped,
/// we can guarantee that the transaction with this state
/// was closed (committed/rolledback)
#[derive(Derivative, Getters, Setters, Clone)]
pub struct TransactionHandle {
  #[getset(get = "pub")]
  serializer: Serializer,
  #[getset(get = "pub")]
  kvstore: Arc<Box<dyn KeyValueStore>>,
  schema_factories: Arc<BTreeMap<String, Arc<SchemaFactory>>>,
  storage_factory_state: Arc<StorageFactoryState>,
  // List if tables locked by this transaction
  locked_tables: Arc<Mutex<Vec<Arc<Table>>>>,
  acquired_locks: Arc<Mutex<Vec<TableSchemaWriteLock>>>,
  lock: TransactionLock,
  // NOTE: this is a hack to pass current query statement to the execution
  // plan so that execution plans can have access to sql data types instead
  // of just datafusion data types; datafusion doesn't support all datatypes
  // and we need to access the query to support custom data types like VECTOR,
  // JSONB, etc
  // TODO: remove this when datafusion support custom data types
  #[getset(get = "pub", set = "pub(crate)")]
  active_statement: Option<Arc<Statement>>,
  #[getset(get = "pub", set = "pub(crate)")]
  is_chained: bool,
}

unsafe impl Send for TransactionHandle {}
unsafe impl Sync for TransactionHandle {}

impl Drop for TransactionHandle {
  fn drop(&mut self) {
    self.storage_factory_state.reduce_active_transaction_count();
  }
}

impl TransactionHandle {
  pub fn new(
    serializer: Serializer,
    kvstore: Arc<Box<dyn KeyValueStore>>,
    schema_factories: Arc<BTreeMap<String, Arc<SchemaFactory>>>,
    storage_factory_state: Arc<StorageFactoryState>,
    locked_tables: Arc<Mutex<Vec<Arc<Table>>>>,
    acquired_locks: Arc<Mutex<Vec<TableSchemaWriteLock>>>,
  ) -> Self {
    Self {
      serializer,
      kvstore,
      schema_factories: schema_factories.clone(),
      storage_factory_state: storage_factory_state.clone(),
      locked_tables,
      acquired_locks,
      lock: TransactionLock {
        lock: Arc::new(AtomicUsize::new(1)),
      },
      active_statement: None,
      is_chained: false,
    }
  }

  // TODO: return mutexlock or some type that is not Send+Sync
  // and gets dropped when it's out of scope so that deadlock error
  // is easily prevented
  // TODO: change this to read/write lock since SELECT that uses more
  // than one table will need more than 1 lock at once
  pub fn lock<'a>(&'a self, exclusive: bool) -> Result<StorageHandler> {
    self.lock.lock(exclusive)?;
    Ok(StorageHandler {
      kv: self.kvstore.clone(),
      serializer: self.serializer.clone(),
      transaction_lock: Some(self.lock.clone()),
    })
  }

  #[inline]
  pub fn closed(&self) -> bool {
    self.lock.closed()
  }

  #[inline]
  pub fn commit(&self) -> Result<()> {
    self.release_lock()?;
    self.kvstore.commit()?;
    Ok(())
  }

  #[inline]
  pub fn rollback(&self) -> Result<()> {
    self.release_lock()?;
    self.kvstore.rollback()?;
    Ok(())
  }

  #[inline]
  fn release_lock(&self) -> Result<()> {
    self.lock.close()?;
    if self.locked_tables.lock().len() > 0 {
      self.storage_factory_state.reload_schema();
    }
    Ok(())
  }

  #[inline]
  pub async fn acquire_table_schema_write_lock(
    &self,
    schema: &str,
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
      .schema_factories
      .get(schema)
      .unwrap()
      .acquire_table_schema_write_lock(table_name)
      .await
  }

  /// Holds the write lock to the table until this transaction is dropped
  #[tracing::instrument(skip(self, table), level = "TRACE")]
  pub fn hold_table_schema_lock(
    &self,
    table: Arc<Table>,
    lock: TableSchemaWriteLock,
  ) -> Result<()> {
    let mut locked_tables = self.locked_tables.lock();
    let existing_locked_table_index = locked_tables
      .iter()
      .position(|table| *table.name == *lock.lock.deref().deref());
    // Remove the table from the locked tables if it exist
    // so that the list will have updated data
    if let Some(index) = existing_locked_table_index {
      locked_tables.remove(index);
    }
    locked_tables.push(table);

    Ok(())
  }

  #[tracing::instrument(skip(self), level = "TRACE")]
  pub fn get_table(&self, schema: &str, name: &str) -> Option<Arc<Table>> {
    // Note: need to check locked_tables first to check if the
    // table was updated by the current transaction but the change
    // hasn't been committed
    self
      .locked_tables
      .lock()
      .iter()
      .find(|locked_table| locked_table.name == name)
      .cloned()
      .or_else(|| {
        self
          .schema_factories
          .get(schema)
          .and_then(|sf| sf.get_table(name))
      })
  }

  #[tracing::instrument(skip(self), level = "TRACE")]
  pub fn table_names(&self, schema: &str) -> Vec<String> {
    self
      .schema_factories
      .get(schema)
      .map(|sf| sf.table_names())
      .unwrap_or_default()
  }
}
