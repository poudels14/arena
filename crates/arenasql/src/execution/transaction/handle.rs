use std::collections::BTreeMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use derivative::Derivative;
use getset::{Getters, Setters};
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
  lock: TransactionLock,
  // NOTE: this is a hack to pass current query statement to the execution
  // plan so that execution plans can have access to sql data types instead
  // of just datafusion data types; datafusion doesn't support all datatypes
  // and we need to access the query to support custom data types like VECTOR,
  // JSONB, etc
  // TODO: remove this when datafusion support custom data types
  #[getset(get = "pub", set = "pub(crate)")]
  active_statement: Option<Arc<Statement>>,
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
  ) -> Self {
    Self {
      serializer,
      kvstore,
      schema_factories: schema_factories.clone(),
      storage_factory_state: storage_factory_state.clone(),
      lock: TransactionLock {
        lock: Arc::new(AtomicUsize::new(1)),
        schema_factories,
        storage_factory_state,
      },
      active_statement: None,
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
    self.lock.close()?;
    self.kvstore.commit()?;
    Ok(())
  }

  #[inline]
  pub fn rollback(&self) -> Result<()> {
    self.lock.close()?;
    self.kvstore.rollback()?;
    Ok(())
  }

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
}
