use std::sync::Arc;

use datafusion::execution::runtime_env::RuntimeEnv as DfRuntimeEnv;

use crate::df::providers::catlog::{
  CatalogListProvider, EmptyCatalogListProvider,
};
use crate::runtime::RuntimeEnv;
use crate::storage::{
  MemoryKeyValueStoreProvider, Serializer, StorageFactory,
  StorageFactoryBuilder,
};

pub struct SessionConfig {
  pub runtime: Arc<RuntimeEnv>,
  pub df_runtime: Arc<DfRuntimeEnv>,
  pub catalog: String,
  pub default_schema: String,
  pub storage_factory: Arc<StorageFactory>,
  pub catalog_list_provider: Arc<dyn CatalogListProvider>,
}

impl Default for SessionConfig {
  fn default() -> Self {
    let default_catalog = "postgres".to_owned();
    let default_schema = "public".to_owned();
    Self {
      runtime: Arc::new(RuntimeEnv::default()),
      catalog: default_catalog.to_owned(),
      default_schema: default_schema.to_owned(),
      df_runtime: Arc::new(DfRuntimeEnv::default()),
      storage_factory: Arc::new(
        StorageFactoryBuilder::default()
          .catalog(default_catalog.clone())
          .serializer(Serializer::VarInt)
          .kv_provider(Arc::new(MemoryKeyValueStoreProvider::default()))
          .build()
          .unwrap(),
      ),
      catalog_list_provider: Arc::new(EmptyCatalogListProvider),
    }
  }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct TaskConfig {
  pub(crate) runtime: Arc<RuntimeEnv>,
}
