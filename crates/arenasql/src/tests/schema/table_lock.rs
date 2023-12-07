use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;

use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn acquire_table_lock_when_creating_new_table() {
  let session = create_session_context();
  let counter = Arc::new(Mutex::new(0));
  let mut tasks = vec![];

  let counter_clone = counter.clone();
  let txn = session.begin_transaction().unwrap();
  tasks.push(tokio::spawn(async move {
    let _ = txn
      .execute_sql(
        r#"CREATE TABLE IF NOT EXISTS unique_column (
          id VARCHAR(50) UNIQUE,
          name TEXT
        )"#,
      )
      .await
      .unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;
    let mut lock = counter_clone.lock().await;
    *lock += 100;

    txn.commit().unwrap();
  }));

  let txn = session.begin_transaction().unwrap();
  let counter_clone = counter.clone();
  tasks.push(tokio::spawn(async move {
    // Note: wait for a bit to make sure another transaction acquired
    // a lock on the table
    tokio::time::sleep(Duration::from_millis(1_00)).await;
    let _ = txn
      .execute_sql(r#"SELECT * FROM unique_column;"#)
      .await
      .unwrap();
    drop(txn);
    assert_eq!(*counter_clone.lock().await, 100);
  }));

  for task in tasks {
    task.await.unwrap();
  }
}
