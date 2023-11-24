use std::sync::Arc;
use std::time::Instant;

use arenasql::df;
use arenasql::runtime::RuntimeEnv;
use arenasql::storage::rocks;
use arenasql::Result;
use dashmap::DashMap;

#[tokio::main]
async fn main() -> Result<()> {
  // let db = arenasql::rocksql::db::SqlDatabase::open(
  //   "./sqldb",
  //   "arena",
  //   "workspace1",
  // )?;

  // // let connection = connection::Connection {
  // //   state: connection::State {
  // //     db_name: "arena".to_owned(),
  // //   },
  // // };

  // let ctx = ExecutionContext::with_state(context::State {
  //   db_name: "arena".to_owned(),
  // });

  // let _ = db
  //   .execute(
  //     &ctx,
  //     r#"CREATE TABLE IF NOT EXISTS workspace1.dqs_nodes (
  //     id VARCHAR(50) UNIQUE,
  //     host VARCHAR(1000)
  //     -- port INTEGER,
  //     -- status VARCHAR(25),
  //     -- started_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  //   )"#,
  //   )
  //   .await
  //   .unwrap();

  // let _ = db
  //   .execute(
  //     &ctx,
  //     r#"
  //       SELECT * FROM (SELECT * FROM workspace1.dqs_nodes WHERE id > 1000)
  //       -- WHERE id > (SELECT id FROM workspace1.dqs_nodes WHERE id > 65000);
  //       "#,
  //   )
  //   .await
  //   .unwrap();

  // let _ = df::execute(
  //   &env,
  //   txn,
  //   r#"SELECT * FROM (
  //     CREATE TABLE IF NOT EXISTS workspace1.dqs_nodes (
  //       id VARCHAR(50) UNIQUE,
  //       host VARCHAR(1000)
  //       -- port INTEGER,
  //       -- status VARCHAR(25),
  //       -- started_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  //     )
  //   );
  //   "#,
  // )
  // .await
  // .unwrap();

  // let _ = df::execute(
  //   &env,
  //   txn,
  //   "INSERT INTO workspace1.dqs_nodes (id) VALUES ('inserted_id_1')",
  // )
  // .await
  // .unwrap();

  let env = RuntimeEnv::new();

  let storage = rocks::RocksStorage::open("db_path", env.clone())?;
  let txn = storage.begin_transaction()?;

  let catlogs = DashMap::new();
  catlogs.insert("arena".to_string(), txn.clone());
  let catlog_list = Arc::new(df::providers::CatalogList {
    runtime: env.clone(),
    catlogs,
  });

  let _ = df::execute(
    &env,
    txn.clone(),
    catlog_list.clone(),
    r#"CREATE TABLE IF NOT EXISTS workspace1.dqs_nodes (
      id VARCHAR(50) UNIQUE,
      host VARCHAR(1000),
      port INTEGER,
      status VARCHAR(25)
      -- vec DECIMAL(8,8)
      -- started_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    )"#,
  )
  .await
  .unwrap();
  // txn.commit().unwrap();

  // let _ = df::execute(
  //   &env,
  //   txn.clone(),
  //   Arc::new(catlog_list.clone()),
  //   // "INSERT INTO workspace1.dqs_nodes VALUES('noice', 'localhost'), ('noice 2', 's')",
  //   // "INSERT INTO workspace1.dqs_nodes(id, host) VALUES('noice', now())",
  //   "INSERT INTO workspace1.dqs_nodes(id, port) VALUES('noice', 9001), ('noice 2', 9002);",
  //   // "INSERT INTO workspace1.dqs_nodes(id) VALUES('noice'), ('noice 2'), ('noice 3');",
  // )
  // .await
  // .unwrap();

  let count = df::execute(
    &env,
    txn.clone(),
    catlog_list.clone(),
    // "SELECT * FROM workspace1.dqs_nodes WHERE id = 1",
    "BEGIN TRANSACTION; SELECT count(*) FROM workspace1.dqs_nodes; COMMIT;", // "SELECT id as id1, port as p1, status, host FROM workspace1.dqs_nodes WHERE port > 9000",
  )
  .await
  .unwrap();
  println!("count = {:?}", count);

  let start = Instant::now();
  for i in 0..20 {
    let txn = storage.begin_transaction().unwrap();
    let _ = df::execute(
      &env,
      txn.clone(),
      catlog_list.clone(),
      &format!("INSERT INTO workspace1.dqs_nodes VALUES('random_id', 'localhost', {}, 'This is the best db out of all!')", i),
    )
    .await
    .unwrap();
    txn.commit().unwrap()
  }
  println!("time taken = {}", start.elapsed().as_millis());

  // let _ = df::execute(
  //   &env,
  //   txn.clone(),
  //   Arc::new(catlog_list.clone()),
  //   // "SELECT * FROM workspace1.dqs_nodes WHERE id = 1",
  //   "SELECT count(*) FROM workspace1.dqs_nodes"
  //   // "SELECT id as id1, port as p1, status, host FROM workspace1.dqs_nodes WHERE port > 9000",
  // )
  // .await
  // .unwrap();

  // txn.commit().unwrap();

  // })
  // .await;

  // let _ = db.execute("SELECT * FROM test.namespace").await.unwrap();

  // let _ = db
  //   .execute("SELECT * FROM dqs_nodes, namespace WHERE namespace.id = 1")
  //   .await
  //   .unwrap();

  Ok(())
}
