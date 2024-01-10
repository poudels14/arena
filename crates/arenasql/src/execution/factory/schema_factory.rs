use std::collections::BTreeMap;
use std::sync::Arc;

use derivative::Derivative;
use derive_builder::Builder;

use crate::execution::locks::{SchemaLocks, TableSchemaWriteLock};
use crate::schema::Table;
use crate::storage::{KeyValueStoreProvider, Serializer, StorageHandler};
use crate::Result;

#[derive(Derivative, Builder)]
#[derivative(Debug)]
pub struct SchemaFactory {
  pub(crate) catalog: Arc<str>,

  pub(crate) schema: Arc<str>,

  #[derivative(Debug = "ignore")]
  kv_store_provider: Arc<dyn KeyValueStoreProvider>,

  #[builder(default = "Serializer::VarInt")]
  pub(crate) serializer: Serializer,

  #[builder(setter(skip), default = "BTreeMap::new()")]
  tables: BTreeMap<String, Arc<Table>>,

  pub(crate) schema_locks: SchemaLocks,
}

impl SchemaFactory {
  #[tracing::instrument(skip(self), level = "TRACE")]
  pub(crate) fn load_all_tables(&mut self) -> Result<()> {
    let kv = self.kv_store_provider.new_transaction()?;
    let storage_handler = StorageHandler {
      kv: Arc::new(kv),
      serializer: self.serializer.clone(),
      transaction_lock: None,
    };

    let all_tables =
      storage_handler.get_all_table_schemas(&self.catalog, &self.schema)?;

    all_tables.into_iter().for_each(|table| {
      self.tables.insert(table.name.to_string(), Arc::new(table));
    });

    Ok(())
  }

  #[tracing::instrument(skip(self), level = "TRACE")]
  pub fn table_names(&self) -> Vec<String> {
    self.tables.values().map(|t| t.name.clone()).collect()
  }

  #[tracing::instrument(skip(self), level = "TRACE")]
  pub fn get_table(&self, name: &str) -> Option<Arc<Table>> {
    self.tables.get(name).map(|kv| kv.clone())
  }

  #[inline]
  #[tracing::instrument(skip(self), level = "trace")]
  /// If the same transaction calls this more than once for the same table,
  /// deadlock occurs. The calling transaction should make sure that it doesn't
  /// already have a lock on this table
  pub async fn acquire_table_schema_write_lock(
    &self,
    table_name: &str,
  ) -> Result<TableSchemaWriteLock> {
    self
      .schema_locks
      .acquire_table_schema_write_lock(table_name)
      .await
  }
}
