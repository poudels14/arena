use std::sync::Arc;

use dashmap::DashMap;
use derivative::Derivative;
use derive_builder::Builder;
use parking_lot::Mutex;

use super::schema::SchemaFactoryBuilder;
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
  lock: Arc<Mutex<usize>>,

  #[builder(setter(skip), default = "DashMap::new()")]
  schemas: DashMap<String, Arc<SchemaFactory>>,
}

impl StorageFactory {
  pub fn being_transaction(&self, schema: &str) -> Result<Transaction> {
    let kv_store = self.kv_provider.new_transaction()?;

    let schema_factory = match self.schemas.get(schema) {
      Some(factory) => factory.value().clone(),
      None => {
        let lock = self.lock.lock();
        // Note: check the map again to make sure another transaction
        // didn't load the schema
        match self.schemas.get(schema) {
          Some(factory) => factory.value().clone(),
          _ => {
            let factory = SchemaFactoryBuilder::default()
              .catalog(self.catalog.clone())
              .schema(schema.to_string())
              .kv_store_provider(self.kv_provider.clone())
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

    Ok(Transaction::new(
      schema_factory,
      Arc::new(kv_store),
      self.serializer.clone(),
    ))
  }
}
