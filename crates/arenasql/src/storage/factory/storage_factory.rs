use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;

use dashmap::DashMap;
use derivative::Derivative;
use derive_builder::Builder;
use parking_lot::Mutex;
use tokio::sync::oneshot;

use super::schema_factory::{SchemaFactory, SchemaFactoryBuilder};
use super::state::{StorageFactoryState, StorageFactoryStateBuilder};
use crate::storage::locks::{SchemaLocks, SchemaLocksBuilder};
use crate::storage::transaction::{
  TransactionBuilder, TransactionStateBuilder,
};
use crate::storage::{KeyValueStoreProvider, Serializer, Transaction};
use crate::{bail, Error, Result};

#[derive(Builder, Derivative)]
#[derivative(Debug)]
pub struct StorageFactory {
  #[builder(setter(custom))]
  catalog: String,

  #[builder(default = "Serializer::VarInt")]
  pub serializer: Serializer,

  #[derivative(Debug = "ignore")]
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
  state: StorageFactoryState,
}

impl StorageFactory {
  pub fn being_transaction(&self, schema: &str) -> Result<Transaction> {
    if self.state.shutdown_triggered() {
      bail!(Error::DatabaseClosed);
    }

    let kv_store = self.kv_provider.new_transaction()?;
    // Clear all schemas for now, it's easier
    if self.state.should_reload_schema() {
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

    let txn_state = TransactionStateBuilder::default()
      .schema_factory(schema_factory)
      .storage_factory_state(self.state.clone())
      .build()
      .unwrap();
    self.state.increase_active_transaction_count();
    Ok(
      TransactionBuilder::default()
        .kv_store(Arc::new(kv_store))
        .serializer(self.serializer.clone())
        .state(Arc::new(txn_state))
        .build()
        .unwrap(),
    )
  }

  /// This waits for all transactions using this storage to complete
  pub async fn graceful_shutdown(&self) -> Result<()> {
    self.state.trigger_shutdown();
    let lock = self.wait_signal.lock().await.take();
    if let Some(signal) = lock {
      signal.await.map_err(|_| {
        Error::InternalError(
          "Error waiting for transactions to close".to_owned(),
        )
      })?;
    }
    Ok(())
  }

  // Note: This is NOT thread safe. So, this shouldn't be called
  // concurrently
  fn get_schema_locks(&self, schema: &str) -> SchemaLocks {
    match self.schema_lock_factories.get(schema) {
      Some(locks) => locks.value().clone(),
      _ => {
        let locks = SchemaLocksBuilder::default().build().unwrap();
        self
          .schema_lock_factories
          .insert(schema.to_owned(), locks.clone());
        locks
      }
    }
  }
}

impl StorageFactoryBuilder {
  pub fn catalog(&mut self, catalog: String) -> &mut Self {
    self.catalog = Some(catalog);

    let (tx, rx) = oneshot::channel();
    self.wait_signal = Some(Arc::new(tokio::sync::Mutex::new(Some(rx))));
    self.state = Some(
      StorageFactoryStateBuilder::default()
        .schema_reload_triggered(Arc::new(AtomicBool::new(false)))
        .shutdown_triggered(Arc::new(AtomicBool::new(false)))
        .shutdown_signal(Arc::new(Mutex::new(Some(tx))))
        .active_transactions_count(Arc::new(AtomicUsize::new(0)))
        .build()
        .unwrap(),
    );
    self
  }
}
