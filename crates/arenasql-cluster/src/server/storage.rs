use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use arenasql::storage::rocks::{self, RocksStorage};
use arenasql::storage::{
  self, KeyValueStoreProvider, MemoryKeyValueStoreProvider, Serializer,
  StorageFactory, StorageFactoryBuilder,
};
use dashmap::DashMap;
use futures::future::join_all;

use crate::error::ArenaClusterResult;
use crate::schema::SYSTEM_CATALOG_NAME;

pub struct ClusterStorageFactory {
  path: PathBuf,
  storages: DashMap<String, Arc<storage::StorageFactory>>,
}

#[derive(Debug, Default)]
pub struct StorageOption {
  /// Rocksdb cache size in MB
  /// Doesn't use cache if it's not passed
  pub cache_size_mb: Option<usize>,
}

impl ClusterStorageFactory {
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
        let key_vaue = match db_name == SYSTEM_CATALOG_NAME {
          true => Some(Arc::new(MemoryKeyValueStoreProvider {})
            as Arc<dyn KeyValueStoreProvider>),
          false => {
            let path = self.path.join("catalogs").join(db_name);
            match path.exists() {
              false => None,
              true => Some(Arc::new(RocksStorage::new_with_cache(
                path,
                options
                  .cache_size_mb
                  .map(|size| rocks::Cache::new_lru_cache(size * 1024 * 1024)),
              )?) as Arc<dyn KeyValueStoreProvider>),
            }
          }
        };

        Ok(key_vaue.map(|kv| {
          let factory = Arc::new(
            StorageFactoryBuilder::default()
              .catalog(db_name.into())
              .serializer(Serializer::VarInt)
              .kv_provider(kv)
              .build()
              .unwrap(),
          );

          self.storages.insert(db_name.to_string(), factory.clone());
          factory
        }))
      }
    }
  }

  pub async fn graceful_shutdown(&self) -> ArenaClusterResult<()> {
    let storages: Vec<(String, Arc<StorageFactory>)> = self
      .storages
      .iter()
      .map(|entry| (entry.key().clone(), entry.value().clone()))
      .collect();

    join_all(
      storages
        .iter()
        .map(|(_, storage)| storage.graceful_shutdown()),
    )
    .await
    .into_iter()
    .map(|r| r.map_err(|e| e.into()))
    .collect::<ArenaClusterResult<Vec<()>>>()?;
    Ok(())
  }
}
