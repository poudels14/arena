use std::sync::Arc;

use datafusion::execution::runtime_env::RuntimeEnv as DfRuntimeEnv;

use super::factory::{StorageFactory, StorageFactoryBuilder};
use super::locks::AdvisoryLocks;
use super::Privilege;
use crate::df::providers::catalog::CatalogListProvider;
use crate::df::providers::NoopCatalogListProvider;
use crate::execution::ExecutionPlanExtension;
use crate::runtime::RuntimeEnv;
use crate::storage::{MemoryKeyValueStoreProvider, Serializer};

pub struct SessionConfig {
  pub runtime: Arc<RuntimeEnv>,
  pub df_runtime: Arc<DfRuntimeEnv>,
  pub catalog: Arc<str>,
  pub schemas: Arc<Vec<String>>,
  pub privilege: Privilege,
  pub storage_factory: Arc<StorageFactory>,
  pub catalog_list_provider: Arc<dyn CatalogListProvider>,
  pub execution_plan_extensions: Arc<Vec<ExecutionPlanExtension>>,
  /// Global advisor locks
  pub advisory_locks: Arc<AdvisoryLocks>,
}

impl Default for SessionConfig {
  fn default() -> Self {
    let default_catalog: Arc<str> = "postgres".into();
    let schemas = Arc::new(vec!["public".to_owned()]);
    Self {
      runtime: Arc::new(RuntimeEnv::default()),
      catalog: default_catalog.clone(),
      schemas: schemas.clone(),
      privilege: Privilege::default(),
      df_runtime: Arc::new(DfRuntimeEnv::default()),
      storage_factory: Arc::new(
        StorageFactoryBuilder::default()
          .catalog(default_catalog.clone())
          .serializer(Serializer::VarInt)
          .kv_provider(Arc::new(MemoryKeyValueStoreProvider::default()))
          .build()
          .unwrap(),
      ),
      catalog_list_provider: Arc::new(NoopCatalogListProvider {}),
      execution_plan_extensions: Arc::new(vec![]),
      advisory_locks: Arc::new(AdvisoryLocks::new()),
    }
  }
}
