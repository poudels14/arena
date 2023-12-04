use std::sync::Arc;

use datafusion::execution::runtime_env::RuntimeEnv as DfRuntimeEnv;

use crate::df::providers::catlog::{
  CatalogListProvider, EmptyCatalogListProvider,
};
use crate::runtime::RuntimeEnv;
use crate::storage::{MemoryStorageProvider, Serializer, StorageProvider};

#[derive(Clone)]
pub struct SessionConfig {
  pub runtime: Arc<RuntimeEnv>,
  pub df_runtime: Arc<DfRuntimeEnv>,
  pub catalog: String,
  pub schema: String,
  pub serializer: Serializer,
  pub storage_provider: Arc<dyn StorageProvider>,
  pub catalog_list_provider: Arc<dyn CatalogListProvider>,
}

impl Default for SessionConfig {
  fn default() -> Self {
    let default_catalog = "postgres".to_owned();
    let default_schema = "public".to_owned();
    Self {
      runtime: Arc::new(RuntimeEnv::default()),
      catalog: default_catalog.to_owned(),
      schema: default_schema.to_owned(),
      df_runtime: Arc::new(DfRuntimeEnv::default()),
      serializer: Serializer::default(),
      storage_provider: Arc::new(MemoryStorageProvider::default()),
      catalog_list_provider: Arc::new(EmptyCatalogListProvider),
    }
  }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct TaskConfig {
  pub(crate) runtime: Arc<RuntimeEnv>,
  pub(crate) serializer: Serializer,
}
