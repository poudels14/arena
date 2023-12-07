use std::sync::Arc;

use dashmap::DashMap;
use derivative::Derivative;
use derive_builder::Builder;
use tokio::runtime::Handle;
use tokio::sync::{OwnedRwLockWriteGuard, RwLock};

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

  #[builder(default = "DashMap::new()")]
  tables: DashMap<String, Arc<Table>>,

  #[builder(default = "DashMap::new()")]
  table_locks: DashMap<String, Arc<RwLock<String>>>,
}

pub struct SchemaLock {
  lock: Option<OwnedRwLockWriteGuard<String>>,
  tables: DashMap<String, Arc<Table>>,
}

impl Drop for SchemaLock {
  fn drop(&mut self) {
    let table =
      OwnedRwLockWriteGuard::downgrade_map(self.lock.take().unwrap(), |n| n);
    self.tables.remove(&*table);
  }
}

impl SchemaFactory {
  pub(crate) fn load_all_tables(&self) -> Result<()> {
    let all_tables = self
      .storage_handler()?
      .get_all_table_schemas(&self.catalog, &self.schema)?;

    all_tables.into_iter().for_each(|table| {
      self.tables.insert(table.name.to_string(), Arc::new(table));
    });

    Ok(())
  }

  pub(crate) fn load_table(&self, name: &str) -> Result<Option<Arc<Table>>> {
    let table = self.storage_handler()?.get_table_schema(
      &self.catalog,
      &self.schema,
      name,
    )?;

    match table {
      Some(table) => {
        let table = Arc::new(table);
        self.tables.insert(table.name.to_string(), table.clone());
        Ok(Some(table))
      }
      _ => Ok(None),
    }
  }

  pub fn get_table(&self, name: &str) -> Option<Arc<Table>> {
    self
      .tables
      .get(name)
      .map(|kv| kv.value().clone())
      .or_else(|| {
        // If the table wasn't found, check if it's locked
        if let Some(lock) = self.table_locks.get(name) {
          tokio::task::block_in_place(|| {
            let _ = lock.blocking_read();
          });
        }
        self.load_table(name).unwrap()
      })
  }

  pub fn lock_table_for_write(&self, name: &str) -> Result<SchemaLock> {
    let owned_lock = tokio::task::block_in_place(|| {
      Handle::current().block_on(async {
        let lock = Arc::new(RwLock::new(name.to_owned()));
        self.table_locks.insert(name.to_owned(), lock.clone());
        lock.write_owned().await
      })
    });

    Ok(SchemaLock {
      lock: Some(owned_lock),
      tables: self.tables.clone(),
    })
  }

  fn storage_handler(&self) -> Result<StorageHandler> {
    let kv = self.kv_store_provider.new_transaction()?;
    Ok(StorageHandler {
      kv: Arc::new(kv),
      lock: Default::default(),
      serializer: self.serializer.clone(),
    })
  }
}
