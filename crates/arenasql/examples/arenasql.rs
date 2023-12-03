use std::sync::Arc;
use std::time::Instant;

use arenasql::records::RecordBatch;
use arenasql::runtime::RuntimeEnv;
use arenasql::storage::rocks;
use arenasql::{Result, SingleCatalogListProvider};
use arenasql::{SessionConfig, SessionContext};
use futures::TryStreamExt;

#[tokio::main]
async fn main() -> Result<()> {
  env_logger::init();

  let runtime = RuntimeEnv::default();
  let storage = Arc::new(rocks::RocksStorage::new_with_cache(
    "_db_path",
    Some(rocks::Cache::new_lru_cache(50 * 1025 * 1024)),
  )?);

  let session_context = SessionContext::with_config(SessionConfig {
    runtime: runtime.into(),
    df_runtime: Default::default(),
    catalog: "arena".to_owned(),
    schema: "workspace1".to_owned(),
    storage_provider: storage.clone(),
    catalog_list_provider: Arc::new(SingleCatalogListProvider::new(
      "arena",
      "workspace1",
    )),
    ..Default::default()
  });

  // ##############################################################################
  // ##############################################################################
  // ##############################################################################

  // let txn = session_context.begin_transaction()?;
  // let _ = txn
  //   .execute_sql(
  //     r#"CREATE TABLE IF NOT EXISTS dqs_nodes_2 (
  //     id VARCHAR(50) UNIQUE,
  //     host VARCHAR(1000) NOT NULL DEFAULT 'localhost',
  //     port INTEGER,
  //     status VARCHAR(25),
  //     remarks TEXT
  //     -- vec DECIMAL(8,8)
  //     -- started_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  //   )"#,
  //   )
  //   .await
  //   .unwrap();

  // let _ = txn
  //   .execute_sql(&format!(
  //     "INSERT INTO dqs_nodes_2(id, host) VALUES('random_id', 'h1'), ('id 2', 'h2')"
  //   ))
  //   .await
  //   .unwrap();

  // let count = txn
  //   .execute_sql("SELECT count(*) FROM dqs_nodes_2;")
  //   .await
  //   .unwrap();
  // println!("count = {:?}", count);
  // txn.rollback().unwrap();
  // panic!();

  // ##############################################################################
  // ##############################################################################
  // ##############################################################################
  let txn = session_context.begin_transaction()?;
  let _ = txn
    .execute_sql(
      r#"CREATE TABLE IF NOT EXISTS dqs_nodes (
      id VARCHAR(50) UNIQUE,
      host VARCHAR(1000) NOT NULL DEFAULT 'localhost',
      port INTEGER,
      status VARCHAR(25)
      -- remarks TEXT
      -- vec DECIMAL(8,8)
      -- started_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    )"#,
    )
    .await
    .unwrap();
  txn.commit().unwrap();

  let txn = session_context.begin_transaction()?;
  let _ = txn
    .execute_sql(&format!(
      "INSERT INTO dqs_nodes(id, host) VALUES('random_id', 'h1'), ('id 2', 'h2')"
    ))
    .await
    .unwrap();
  txn.commit().unwrap();

  println!("-------------------------------------------------------------");

  // let start = Instant::now();
  // for i in 0..5_000 {
  //   let txn = session_context.begin_transaction()?;

  //   let _ = txn.execute_sql(
  //     &format!("INSERT INTO workspace1.dqs_nodes VALUES('random_id', 'localhost', {}, 'This is the best db out of all!')", i),
  //   )
  //   .await
  //   .unwrap();
  //   txn.commit().unwrap()
  // }
  // println!("time taken = {}", start.elapsed().as_millis());

  println!("-------------------------------------------------------------");

  // ##############################################################################
  // ##############################################################################
  // ##############################################################################

  // let start = Instant::now();
  // for i in 0..2_000 {
  //   let txn = session_context.begin_transaction()?;

  //   let _ = txn.execute_sql(
  //     &format!("INSERT INTO workspace1.dqs_nodes VALUES('random_id', 'localhost', {}, 'This is the best db out of all!')", i),
  //   )
  //   .await
  //   .unwrap();
  //   txn.rollback().unwrap();
  //   // txn.commit().unwrap();
  // }
  // println!("time taken = {}", start.elapsed().as_millis());

  println!("-------------------------------------------------------------");

  let start = Instant::now();
  let txn = session_context.begin_transaction()?;
  let result = txn
    .execute_sql(
      // "SELECT * FROM workspace1.dqs_nodes WHERE id = 1",
      "SELECT count(id) FROM dqs_nodes;",
      // "SELECT id as id1, port as p1, status, host FROM workspace1.dqs_nodes WHERE port > 9000",
    )
    .await
    .unwrap();

  let count: Vec<RecordBatch> = result.stream.try_collect().await.unwrap();
  println!("count = {:?}", count);
  println!("time taken = {}", start.elapsed().as_millis());
  println!("-------------------------------------------------------------");

  // println!("-------------------------------------------------------------");

  // let start = Instant::now();
  // let txn = session_context.begin_transaction()?;
  // let result = txn
  //   .execute_sql(
  // "SELECT * FROM workspace1.dqs_nodes WHERE id = 1",
  // "SELECT count(*) FROM workspace1.dqs_nodes WHERE port > 1;",
  // "DELETE FROM dqs_nodes WHERE 1 = 1"
  // "UPDATE dqs_nodes SET host = 'new_host' WHERE id = '123'",
  //     "SELECT id as id1, port as p1, status, host FROM workspace1.dqs_nodes WHERE port > 9000",
  //   )
  //   .await
  //   .unwrap();
  // let count: Vec<RecordBatch> = result.stream.try_collect().await.unwrap();
  // println!("count = {:?}", count);
  // println!("time taken = {}", start.elapsed().as_millis());
  // println!("-------------------------------------------------------------");

  // storage.compact_and_flush().unwrap();
  // println!("DB SIZE = {}", storage.get_db_size().unwrap());

  println!("-------------------------------------------------------------");

  let start = Instant::now();
  // let stmst: Vec<Box<sqlparser::ast::Statement>> =
  //   parser::parse(&format!("SELECT * FROM dqs_nodes"))
  //     .unwrap()
  //     .into_iter()
  //     .map(|s| Box::new(s))
  //     .collect();
  // for i in 0..2_000 {

  let txn = session_context.begin_transaction()?;
  let _ = &txn
    .execute_sql(
      &format!("SELECT * FROM dqs_nodes"), // &format!("SELECT count(id) FROM dqs_nodes"),
    )
    .await
    .unwrap();
  // txn.rollback().unwrap();
  txn.commit().unwrap();
  println!("time taken = {}", start.elapsed().as_millis());

  println!("-------------------------------------------------------------");

  Ok(())
}
