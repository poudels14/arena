use std::str::FromStr;
use std::sync::{Arc, Once};

use tempdir::TempDir;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::prelude::*;
use tracing_tree::HierarchicalLayer;

use crate::execution::factory::StorageFactoryBuilder;
use crate::execution::{
  Privilege, SessionConfig, SessionContext, DEFAULT_SCHEMA_NAME,
};
use crate::runtime::RuntimeEnv;
use crate::storage::rocks;
use crate::SingleCatalogListProvider;

mod datatype;
mod delete_query;
mod drop_table;
mod insert_query;
mod schema;
mod select_query;
mod statement;
mod transaction;
mod update_query;
mod vectors;

#[macro_export]
macro_rules! execute_query {
  ($txn:tt, $query:expr) => {
    $txn.execute_sql(&format!($query)).await
  };
}

static INIT_LOGGER: Once = Once::new();

pub(super) fn create_session_context() -> SessionContext {
  INIT_LOGGER.call_once(|| {
    let subscriber = tracing_subscriber::registry()
      .with(
        tracing_subscriber::filter::EnvFilter::builder()
          .from_env_lossy()
          // Note(sagar): filter out noisy logs
          .add_directive(Directive::from_str("swc_=OFF").unwrap())
          .add_directive(Directive::from_str("tokio_=OFF").unwrap()),
      )
      .with(
        HierarchicalLayer::default()
          .with_indent_amount(2)
          .with_thread_names(true),
      );
    tracing::subscriber::set_global_default(subscriber).unwrap();
  });

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
  .unwrap()
}
