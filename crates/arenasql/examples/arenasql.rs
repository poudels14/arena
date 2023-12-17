#![allow(unused)]
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use arenasql::runtime::RuntimeEnv;
use arenasql::storage::{rocks, StorageFactoryBuilder};
use arenasql::{Result, SingleCatalogListProvider};
use arenasql::{SessionConfig, SessionContext};

#[tokio::main]
async fn main() -> Result<()> {
  env_logger::init();

  let runtime = RuntimeEnv::default();
  let storage = Arc::new(rocks::RocksStorage::new_with_cache(
    Path::new("_db_path").to_path_buf(),
    Some(rocks::Cache::new_lru_cache(50 * 1025 * 1024)),
  )?);

  let storage_factory = Arc::new(
    StorageFactoryBuilder::default()
      .catalog("arena".to_owned())
      .kv_provider(storage.clone())
      .build()
      .unwrap(),
  );

  let session_context = SessionContext::with_config(SessionConfig {
    runtime: runtime.into(),
    df_runtime: Default::default(),
    catalog: "arena".to_owned(),
    default_schema: "workspace1".to_owned(),
    storage_factory: storage_factory.clone(),
    catalog_list_provider: Arc::new(SingleCatalogListProvider::new(
      "arena",
      "workspace1",
    )),
    ..Default::default()
  });

  {
    let txn = session_context.begin_transaction().unwrap();

    let _res = txn
      .execute_sql(
        r#"CREATE TABLE IF NOT EXISTS vector_table (
          id VARCHAR(50),
          embed VECTOR(4)
        )"#,
      )
      .await
      .unwrap();

    let res = txn
      .execute_sql(r#"SELECT count(id) FROM test_table WHERE id > $1"#)
      .await
      .unwrap();
    println!("count = {:?}", res.get_count().await.unwrap());
    drop(txn);
  }

  storage_factory.graceful_shutdown().await.unwrap();

  Ok(())
}
