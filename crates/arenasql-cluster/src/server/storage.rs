use std::fs::{self, read_dir};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use arenasql::execution::factory::{StorageFactory, StorageFactoryBuilder};
use arenasql::rocks::{BackupEngine, BackupEngineOptions, Env};
use arenasql::storage::rocks::{self, RocksStorage};
use arenasql::storage::{
  KeyValueStoreProvider, MemoryKeyValueStoreProvider, Serializer,
};
use arenasql::Result;
use dashmap::DashMap;
use futures::future::join_all;
use getset::{Getters, Setters};
use tracing::info;
use tokio::task::JoinHandle;

use crate::error::ArenaClusterResult;
use crate::schema::SYSTEM_CATALOG_NAME;

#[derive(Getters)]
pub struct ClusterStorageFactory {
  #[getset(get = "pub")]
  options: StorageOption,
  storages: DashMap<String, Arc<StorageFactory>>,
}

#[derive(Debug, Default, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
pub struct StorageOption {
  root_dir: Arc<PathBuf>,

  /// Directory to backup database to
  /// If set, all the database that were opened by the cluster will be
  /// backed up to that directory periodically
  backup_dir: Option<PathBuf>,

  /// Directory to put a checkpoint of the databases to
  /// When cluster is terminated, all the databases that were opened will
  /// be checkpointed to that directory
  checkpoint_dir: Option<PathBuf>,

  /// Rocksdb cache size in MB
  /// Doesn't use cache if it's not passed
  cache_size_mb: Option<usize>,
}

impl ClusterStorageFactory {
  pub fn new(options: StorageOption) -> Self {
    if !options.root_dir.exists() {
      fs::create_dir_all(&options.root_dir.as_ref()).expect(&format!(
        "Failed to create database directory: {:?}",
        options.root_dir
      ));
    }
    Self {
      options,
      storages: DashMap::new(),
    }
  }

  pub fn get_catalog(
    &self,
    db_name: &str,
  ) -> Result<Option<Arc<StorageFactory>>> {
    let storage = self.storages.get(db_name);
    match storage {
      Some(storage) => Ok(Some(storage.value().clone())),
      None => {
        let key_vaue = match db_name == SYSTEM_CATALOG_NAME {
          true => Some(Arc::new(MemoryKeyValueStoreProvider {})
            as Arc<dyn KeyValueStoreProvider>),
          false => {
            let cache = self
              .options
              .cache_size_mb
              .map(|size| rocks::Cache::new_lru_cache(size * 1024 * 1024));
            let db_dir = self.options.root_dir.join("catalogs").join(db_name);
            let rocks_storage = match db_dir.exists() {
              false => {
                if let Some(checkpoint_dir) = &self.options.checkpoint_dir {
                  let catalog_checkpoint_dir = checkpoint_dir.join(db_name);
                  let max_checkpoint = read_dir(&catalog_checkpoint_dir)
                    .unwrap()
                    .filter_map(|dir| {
                      pathdiff::diff_paths(
                        &dir.unwrap().path(),
                        &catalog_checkpoint_dir,
                      )
                      .and_then(|p| {
                        let timestamp_str = p.to_str();
                        timestamp_str.and_then(|str| str.parse::<u64>().ok())
                      })
                    })
                    .max();
                  match max_checkpoint {
                    Some(checkpoint_timestamp) => {
                      info!(
                        "Loading catalog \"{}\" from checkpoint: {:?}",
                        db_name, checkpoint_timestamp
                      );
                      let start = Instant::now();
                      let db = Some(RocksStorage::load_from_backup(
                        catalog_checkpoint_dir
                          .join(format!("{}", checkpoint_timestamp))
                          .to_str()
                          .unwrap(),
                        db_dir,
                        cache,
                      )?);

                      info!(
                        "Time taken to load catalog \"{}\" from checkpoint: {}s",
                        db_name,
                        start.elapsed().as_secs()
                      );

                      db
                    }
                    _ => None,
                  }
                } else {
                  None
                }
              }
              true => Some(RocksStorage::new_with_cache(db_dir, cache)?),
            };
            rocks_storage.map(|storage| {
              Arc::new(storage) as Arc<dyn KeyValueStoreProvider>
            })
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
    let timetamp = SystemTime::now();
    let storages: Vec<JoinHandle<()>> = self
      .storages
      .iter()
      .map(|entry| {
        let pair = entry.pair();
        let (catalog, storage) = (pair.0.clone(), pair.1.clone());
        let checkpoint_dir = self.options.checkpoint_dir.clone();
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
    storage: &StorageFactory,
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

    let started_at = Instant::now();

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

      info!(
        "Catalog \"{}\" checkpoint saved {:?}, time taken = {}s",
        catalog_name,
        catalog_checkpoint_dir,
        started_at.elapsed().as_secs(),
      );
    }

    Ok(())
  }
}
