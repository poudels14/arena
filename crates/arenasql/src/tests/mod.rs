use std::sync::Arc;

use tempdir::TempDir;

use crate::runtime::RuntimeEnv;
use crate::storage::{rocks, StorageFactoryBuilder};
use crate::{SessionConfig, SessionContext, SingleCatalogListProvider};

mod datatype;
mod insert;
mod schema;
mod select;

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

  let catalog = "test";
  let schema = "public";
  SessionContext::with_config(SessionConfig {
    runtime: runtime.into(),
    df_runtime: Default::default(),
    catalog: catalog.to_owned(),
    default_schema: schema.to_owned(),
    storage_factory: Arc::new(
      StorageFactoryBuilder::default()
        .catalog(catalog.to_owned())
        .kv_provider(storage)
        .build()
        .unwrap(),
    ),
    catalog_list_provider: Arc::new(SingleCatalogListProvider::new(
      catalog, schema,
    )),
    ..Default::default()
  })
}
