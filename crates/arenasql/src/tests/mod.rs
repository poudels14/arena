use std::sync::Arc;

use tempdir::TempDir;

use crate::execution::{
  Privilege, SessionConfig, SessionContext, DEFAULT_SCHEMA_NAME,
};
use crate::runtime::RuntimeEnv;
use crate::storage::{rocks, StorageFactoryBuilder};
use crate::SingleCatalogListProvider;

mod datatype;
mod delete_query;
mod insert_query;
mod schema;
mod select_query;
mod update_query;

#[macro_export]
macro_rules! execute_query {
  ($txn:tt, $query:expr) => {
    $txn.execute_sql(&format!($query)).await
  };
}

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

  let catalog: Arc<str> = "test".into();
  let schemas = Arc::new(vec![DEFAULT_SCHEMA_NAME.to_string()]);
  SessionContext::new(
    SessionConfig {
      runtime: runtime.into(),
      df_runtime: Default::default(),
      catalog: catalog.clone(),
      schemas: schemas.clone(),
      storage_factory: Arc::new(
        StorageFactoryBuilder::default()
          .catalog(catalog)
          .kv_provider(storage)
          .build()
          .unwrap(),
      ),
      catalog_list_provider: Arc::new(SingleCatalogListProvider::new()),
      privilege: Privilege::SUPER_USER,
      ..Default::default()
    },
    Default::default(),
  )
}
