use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::Arc;

use derivative::Derivative;
use derive_builder::Builder;
use parking_lot::{Mutex, RwLock};

use crate::schema::Table;
use crate::storage::locks::{SchemaLocks, TableSchemaWriteLock};
use crate::storage::{KeyValueStoreProvider, Serializer, StorageHandler};
use crate::Result;

#[derive(Derivative, Builder)]
#[derivative(Debug)]
pub struct SchemaFactory {
  pub(crate) catalog: String,

  pub(crate) schema: String,

  #[derivative(Debug = "ignore")]
  kv_store_provider: Arc<dyn KeyValueStoreProvider>,

  #[builder(default = "Serializer::VarInt")]
  pub(crate) serializer: Serializer,

  #[builder(setter(skip), default = "BTreeMap::new()")]
  tables: BTreeMap<String, Arc<Table>>,

  #[builder(setter(skip), default = "Arc::new(Mutex::new(vec![]))")]
  pub(crate) locked_tables: Arc<Mutex<Vec<TableSchemaWriteLock>>>,

  #[builder(setter(skip), default = "Arc::new(RwLock::new(vec![]))")]
  table_locks: Arc<RwLock<Vec<String>>>,

  pub(crate) schema_locks: SchemaLocks,
}

impl SchemaFactory {
  pub(crate) fn load_all_tables(&mut self) -> Result<()> {
    let kv = self.kv_store_provider.new_transaction()?;
    let storage_handler = StorageHandler {
      kv: Arc::new(kv),
      serializer: self.serializer.clone(),
      transaction_state: None,
    };

    let all_tables =
      storage_handler.get_all_table_schemas(&self.catalog, &self.schema)?;

    all_tables.into_iter().for_each(|table| {
      self.tables.insert(table.name.to_string(), Arc::new(table));
    });

    Ok(())
  }

  pub fn table_names(&self) -> Vec<String> {
    let mut tables: Vec<String> = self
      .tables
      .values()
      .map(|t| t.name.clone())
      .chain(
        self
          .locked_tables
          .lock()
          .iter()
          .filter_map(|t| t.table.as_ref().map(|t| t.name.clone())),
      )
      .collect();
    tables.dedup();
    tables
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
      .or_else(|| self.tables.get(name).map(|kv| kv.clone()))
  }

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
}
