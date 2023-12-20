use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use arenasql::rocks::{BackupEngine, BackupEngineOptions, Env};
use arenasql::storage::rocks::{self, RocksStorage};
use arenasql::storage::{
  self, KeyValueStoreProvider, MemoryKeyValueStoreProvider, Serializer,
  StorageFactoryBuilder,
};
use dashmap::DashMap;
use futures::future::join_all;
use log::info;
use tokio::task::JoinHandle;

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

  pub async fn graceful_shutdown(
    &self,
    checkpoint_dir: Option<PathBuf>,
  ) -> ArenaClusterResult<()> {
    let timetamp = SystemTime::now();
    let storages: Vec<JoinHandle<()>> = self
      .storages
      .iter()
      .map(|entry| {
        let pair = entry.pair();
        let (catalog, storage) = (pair.0.clone(), pair.1.clone());
        let checkpoint_dir = checkpoint_dir.clone();
        tokio::spawn(async move {
          let _ = storage.graceful_shutdown().await;
          if let Some(checkpoint_dir) = checkpoint_dir {
            let res = Self::checkpoint_catalog(
              checkpoint_dir,
              storage.as_ref(),
              &catalog,
              &timetamp,
            )
            .await;

            // Print the graceful shutdown error and
            // ignore it since it's inconsequential
            if let Err(err) = res {
              eprintln!(
                "Error checkpointing catalog \"{}\": {:?}",
                catalog, err
              );
            }
          }
        })
      })
      .collect();

    // Ignore join error :shrug:
    let _ = join_all(storages).await;
    Ok(())
  }

  async fn checkpoint_catalog(
    checkpoint_dir: PathBuf,
    storage: &storage::StorageFactory,
    catalog_name: &str,
    timetamp: &SystemTime,
  ) -> ArenaClusterResult<()> {
    let checkpoint_dir = checkpoint_dir.join(catalog_name);
    if !checkpoint_dir.exists() {
      std::fs::create_dir_all(&checkpoint_dir)?;
    }
    let catalog_checkpoint_dir = checkpoint_dir.join(format!(
      "{}",
      timetamp.duration_since(UNIX_EPOCH).unwrap().as_millis()
    ));

    info!(
      "Checkpointing catalog \"{}\" to {:?}",
      catalog_name, catalog_checkpoint_dir
    );

    if let Some(rocks) = storage
      .kv_provider()
      .as_any()
      .downcast_ref::<RocksStorage>()
    {
      let backup_opts =
        BackupEngineOptions::new(catalog_checkpoint_dir.to_str().unwrap())?;
      let mut env = Env::new()?;
      env.set_background_threads(4);

      let mut engine = BackupEngine::open(&backup_opts, &env)?;
      // TODO: call fsync somehow
      engine.create_new_backup(rocks.db())?;
    }

    Ok(())
  }
}
