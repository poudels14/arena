use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use dashmap::DashMap;
use derivative::Derivative;
use derive_builder::Builder;
use parking_lot::Mutex;

use super::locks::{SchemaLocks, SchemaLocksBuilder};
use super::schema_factory::SchemaFactoryBuilder;
use super::transaction::TransactionBuilder;
use super::{KeyValueStoreProvider, SchemaFactory, Serializer, Transaction};
use crate::Result;

#[derive(Builder, Derivative)]
#[derivative(Debug)]
pub struct StorageFactory {
  catalog: String,

  #[builder(default = "Serializer::VarInt")]
  pub serializer: Serializer,

  #[derivative(Debug = "ignore")]
  kv_provider: Arc<dyn KeyValueStoreProvider>,

  #[builder(setter(skip), default = "Arc::new(Mutex::new(0))")]
  factory_lock: Arc<Mutex<usize>>,

  #[builder(setter(skip), default = "DashMap::new()")]
  schemas: DashMap<String, Arc<SchemaFactory>>,

  /// If this is set to true, another transaction will load table
  /// schemas from store to get the updated copy. This is used to
  /// trigger reload when table schemas are updated
  #[builder(setter(skip), default = "Arc::new(AtomicBool::new(false))")]
  schema_reload_flag: Arc<AtomicBool>,

  #[builder(setter(skip), default = "DashMap::new()")]
  schema_lock_factories: DashMap<String, SchemaLocks>,
}

impl StorageFactory {
  pub fn being_transaction(&self, schema: &str) -> Result<Transaction> {
    let kv_store = self.kv_provider.new_transaction()?;

    // Clear all schemas for now, it's easier
    if self.schema_reload_flag.load(Ordering::Acquire) {
      self.schemas.clear();
    }

    let schema_factory = match self.schemas.get(schema) {
      Some(factory) => factory.value().clone(),
      None => {
        let lock = self.factory_lock.lock();
        // Note: check the map again to make sure another transaction
        // didn't load the schema
        match self.schemas.get(schema) {
          Some(factory) => factory.value().clone(),
          _ => {
            let mut factory = SchemaFactoryBuilder::default()
              .catalog(self.catalog.clone())
              .schema(schema.to_string())
              .kv_store_provider(self.kv_provider.clone())
              .schema_reload_flag(self.schema_reload_flag.clone())
              .schema_locks(self.get_schema_locks(&schema))
              .build()
              .unwrap();
            factory.load_all_tables()?;
            let factory = Arc::new(factory);
            self.schemas.insert(schema.to_owned(), factory.clone());
            drop(lock);
            factory
          }
        }
      }
    };

    Ok(
      TransactionBuilder::default()
        .schema_factory(schema_factory)
        .kv_store(Arc::new(kv_store))
        .serializer(self.serializer.clone())
        .build()
        .unwrap(),
    )
  }

  // Note: This is NOT thread safe. So, this shouldn't be called
  // concurrently
  fn get_schema_locks(&self, schema: &str) -> SchemaLocks {
    match self.schema_lock_factories.get(schema) {
      Some(locks) => locks.value().clone(),
      _ => {
        let locks = SchemaLocksBuilder::default()
          .schema_reload_flag(self.schema_reload_flag.clone())
          .build()
          .unwrap();
        self
          .schema_lock_factories
          .insert(schema.to_owned(), locks.clone());
        locks
      }
    }
  }
}
