use std::collections::BTreeMap;
use std::sync::Arc;

use dashmap::DashMap;
use derivative::Derivative;
use derive_builder::Builder;
use getset::Getters;
use parking_lot::Mutex;
use tokio::sync::oneshot;

use super::schema_factory::{SchemaFactory, SchemaFactoryBuilder};
use super::state::StorageFactoryState;
use crate::execution::locks::{SchemaLocks, SchemaLocksBuilder};
use crate::execution::TransactionHandle;
use crate::storage::{KeyValueStoreProvider, Serializer};
use crate::{bail, Error, Result};

#[derive(Builder, Derivative, Getters)]
#[derivative(Debug)]
pub struct StorageFactory {
  #[builder(setter(custom))]
  catalog: Arc<str>,

  #[builder(default = "Serializer::VarInt")]
  pub serializer: Serializer,

  #[derivative(Debug = "ignore")]
  #[getset(get = "pub")]
  kv_provider: Arc<dyn KeyValueStoreProvider>,

  #[builder(setter(skip), default = "Arc::new(Mutex::new(0))")]
  factory_lock: Arc<Mutex<usize>>,

  #[builder(setter(skip), default = "DashMap::new()")]
  schemas: DashMap<String, Arc<SchemaFactory>>,

  #[builder(setter(skip), default = "DashMap::new()")]
  schema_lock_factories: DashMap<String, SchemaLocks>,

  #[builder(private)]
  wait_signal: Arc<tokio::sync::Mutex<Option<oneshot::Receiver<()>>>>,

  #[builder(private)]
  state: Arc<StorageFactoryState>,
}

impl StorageFactory {
  #[tracing::instrument(skip(self), level = "TRACE")]
  pub fn being_transaction(
    &self,
    schemas: Arc<Vec<String>>,
  ) -> Result<TransactionHandle> {
    if self.state.shutdown_triggered() {
      bail!(Error::DatabaseClosed);
    }

    // Clear all schemas for now, it's easier
    if self.state.should_reload_schema() {
      self.schemas.clear();
    }

    let kvstore = self.kv_provider.new_transaction()?;
    let schema_factories = schemas
      .iter()
      .map(|schema| match self.schemas.get(schema) {
        Some(factory) => Ok((schema.to_owned(), factory.value().clone())),
        None => {
          let lock = self.factory_lock.lock();
          // Note: check the map again to make sure another transaction
          // didn't load the schema
          match self.schemas.get(schema) {
            Some(factory) => Ok((schema.to_owned(), factory.value().clone())),
            _ => {
              let mut factory = SchemaFactoryBuilder::default()
                .catalog(self.catalog.clone())
                .schema(schema.as_str().into())
                .kv_store_provider(self.kv_provider.clone())
                .schema_locks(self.get_schema_locks(&schema))
                .build()
                .unwrap();
              factory.load_all_tables()?;
              let factory = Arc::new(factory);
              self.schemas.insert(schema.to_owned(), factory.clone());
              drop(lock);
              Ok((schema.to_owned(), factory))
            }
          }
        }
      })
      .collect::<Result<BTreeMap<String, Arc<SchemaFactory>>>>()?;

    Ok(TransactionHandle::new(
      self.serializer.clone(),
      Arc::new(kvstore),
      schema_factories.into(),
      self.state.clone(),
      Arc::new(Mutex::new(vec![])),
      Arc::new(Mutex::new(vec![])),
    ))
  }

  /// This waits for all transactions using this storage to complete
  pub async fn graceful_shutdown(&self) -> Result<()> {
    self.state.trigger_shutdown();
    let lock = self.wait_signal.lock().await.take();
    if let Some(signal) = lock {
      if self.state.active_transactions() > 0 {
        signal.await.map_err(|_| {
          Error::InternalError(
            "Error waiting for transactions to close".to_owned(),
          )
        })?;
      }
    }
    Ok(())
  }

  // Note: This is NOT thread safe. So, this shouldn't be called
  // concurrently
  fn get_schema_locks(&self, schema: &str) -> SchemaLocks {
    match self.schema_lock_factories.get(schema) {
      Some(locks) => locks.value().clone(),
      _ => {
        let locks = SchemaLocksBuilder::default()
          .schema(schema.into())
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

impl StorageFactoryBuilder {
  pub fn catalog(&mut self, catalog: Arc<str>) -> &mut Self {
    self.catalog = Some(catalog);

    let (tx, rx) = oneshot::channel();
    self.wait_signal = Some(Arc::new(tokio::sync::Mutex::new(Some(rx))));
    self.state = Some(Arc::new(StorageFactoryState::new(Some(tx))));
    self
  }
}
