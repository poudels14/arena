use std::sync::Arc;

use tempdir::TempDir;

use crate::runtime::RuntimeEnv;
use crate::storage::rocks;
use crate::{SessionConfig, SessionContext, SingleCatalogListProvider};

mod unique_column;

pub(super) fn create_session_context() -> SessionContext {
  let runtime = RuntimeEnv::default();

  let db_path = TempDir::new("arenasql").unwrap();
  let storage = Arc::new(
    rocks::RocksStorage::new_with_cache(
      db_path.into_path(),
      Some(rocks::Cache::new_lru_cache(50 * 1025 * 1024)),
    )
    .unwrap(),
  );

  let catalog = "test_catalog";
  let schema = "test_schema";
  SessionContext::with_config(SessionConfig {
    runtime: runtime.into(),
    df_runtime: Default::default(),
    catalog: catalog.to_owned(),
    schema: schema.to_owned(),
    storage_provider: storage.clone(),
    catalog_list_provider: Arc::new(SingleCatalogListProvider::new(
      catalog, schema,
    )),
    ..Default::default()
  })
}
