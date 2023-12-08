use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use arenasql::storage::rocks::{self, RocksStorage};
use arenasql::storage::{self, Serializer, StorageFactoryBuilder};
use dashmap::DashMap;

use crate::error::{ArenaClusterError, ArenaClusterResult};

pub struct StorageFactory {
  path: PathBuf,
  storages: DashMap<String, Arc<storage::StorageFactory>>,
}

impl StorageFactory {
  pub fn new(path: PathBuf) -> Self {
    if !path.exists() {
      fs::create_dir_all(&path)
        .expect(&format!("Failed to create database directory: {:?}", path));
    }
    Self {
      path,
      storages: DashMap::new(),
    }
  }

  pub fn get(
    &self,
    db_name: &str,
  ) -> ArenaClusterResult<Option<Arc<storage::StorageFactory>>> {
    let storage = self.storages.get(db_name);
    match storage {
      Some(storage) => Ok(Some(storage.value().clone())),
      None => {
        let path = self.path.join(db_name);
        match path.exists() {
          false => Ok(None),
          true => {
            let kv = Arc::new(
              RocksStorage::new_with_cache(
                path,
                // TODO: pass this as config
                Some(rocks::Cache::new_lru_cache(50 * 1025 * 1024)),
              )
              .map_err(|_| ArenaClusterError::StorageError)?,
            );

            let factory = Arc::new(
              StorageFactoryBuilder::default()
                .catalog(db_name.to_owned())
                .serializer(Serializer::VarInt)
                .kv_provider(kv)
                .build()
                .unwrap(),
            );

            self.storages.insert(db_name.to_string(), factory.clone());
            Ok(Some(factory))
          }
        }
      }
    }
  }
}
