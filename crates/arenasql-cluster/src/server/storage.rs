use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use arenasql::storage::rocks::{self, RocksStorage};
use arenasql::storage::{self, Serializer, StorageFactoryBuilder};
use dashmap::DashMap;

use crate::error::ArenaClusterResult;

pub struct StorageFactory {
  path: PathBuf,
  storages: DashMap<String, Arc<storage::StorageFactory>>,
}

#[derive(Debug, Default)]
pub struct StorageOption {
  /// Rocksdb cache size in MB
  /// Doesn't use cache if it's not passed
  pub cache_size_mb: Option<usize>,
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

  pub fn get_catalog(
    &self,
    db_name: &str,
    options: StorageOption,
  ) -> ArenaClusterResult<Option<Arc<storage::StorageFactory>>> {
    let storage = self.storages.get(db_name);
    match storage {
      Some(storage) => Ok(Some(storage.value().clone())),
      None => {
        let path = self.path.join("catalogs").join(db_name);
        match path.exists() {
          false => Ok(None),
          true => {
            let kv = Arc::new(RocksStorage::new_with_cache(
              path,
              options
                .cache_size_mb
                .map(|size| rocks::Cache::new_lru_cache(size * 1024 * 1024)),
            )?);

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
