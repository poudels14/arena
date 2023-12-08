use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use derivative::Derivative;
use derive_builder::Builder;

use super::locks::SchemaLocks;
use super::{KeyValueStoreProvider, Serializer, StorageHandler};
use crate::schema::Table;
use crate::Result;

#[derive(Derivative, Builder)]
#[derivative(Debug)]
pub struct SchemaFactory {
  pub(crate) catalog: String,

  pub(crate) schema: String,

  #[derivative(Debug = "ignore")]
  kv_store_provider: Arc<dyn KeyValueStoreProvider>,

  #[builder(default = "Serializer::VarInt")]
  pub serializer: Serializer,

  #[builder(setter(skip), default = "HashMap::new()")]
  tables: HashMap<String, Arc<Table>>,

  schema_reload_flag: Arc<AtomicBool>,

  pub(super) schema_locks: SchemaLocks,
}

impl SchemaFactory {
  pub(crate) fn load_all_tables(&mut self) -> Result<()> {
    let kv = self.kv_store_provider.new_transaction()?;
    let storage_handler = StorageHandler {
      kv: Arc::new(kv),
      lock: Default::default(),
      serializer: self.serializer.clone(),
    };

    let all_tables =
      storage_handler.get_all_table_schemas(&self.catalog, &self.schema)?;

    all_tables.into_iter().for_each(|table| {
      self.tables.insert(table.name.to_string(), Arc::new(table));
    });

    Ok(())
  }

  pub fn table_names(&self) -> Vec<String> {
    self.tables.values().map(|t| t.name.clone()).collect()
  }

  /// Note: if the table was created in the current transaction
  /// and the transaction hasn't been committed yet, this
  /// will acquire the table lock and it SHOULD be AVIOIDED
  pub fn get_table(&self, name: &str) -> Option<Arc<Table>> {
    self.tables.get(name).map(|kv| kv.clone())
  }
}
