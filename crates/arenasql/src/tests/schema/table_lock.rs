use std::time::Duration;

use crate::execute_query;
use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn another_txn_shouldnt_find_table_until_create_table_is_committed() {
  let session = create_session_context();
  let mut tasks = vec![];

  let txn = session.begin_transaction().unwrap();
  tasks.push(tokio::spawn(async move {
    let _ = execute_query!(
      txn,
      r#"CREATE TABLE IF NOT EXISTS unique_column (
          id VARCHAR(50) UNIQUE,
          name TEXT
        )"#
    )
    .unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;
    txn.commit().unwrap();
  }));

  let txn = session.begin_transaction().unwrap();
  tasks.push(tokio::spawn(async move {
    let res = execute_query!(txn, r#"SELECT * FROM unique_column;"#);
    drop(txn);
    assert!(
      res.is_err(),
      "Second transaction was successfult but it shouldn't be"
    );
  }));

  for task in tasks {
    task.await.unwrap();
  }
}

#[tokio::test(flavor = "multi_thread")]
async fn another_txn_should_find_table_after_create_table_is_committed() {
  let session = create_session_context();
  let txn = session.begin_transaction().unwrap();
  let task1 = tokio::spawn(async move {
    let _ = execute_query!(
      txn,
      r#"CREATE TABLE IF NOT EXISTS unique_column (
          id VARCHAR(50) UNIQUE,
          name TEXT
        )"#
    )
    .unwrap();
    txn.commit().unwrap();
  });

  // Wait until the first transction is committed, otherwise the second
  // transaction can't find the table
  tokio::time::sleep(Duration::from_millis(10)).await;

  let txn = session.begin_transaction().unwrap();
  let task2 = tokio::spawn(async move {
    let res = execute_query!(txn, r#"SELECT * FROM unique_column;"#);
    drop(txn);
    res
  });

  task1.await.unwrap();
  assert!(
    task2.await.unwrap().is_ok(),
    "Second transaction didn't find the new table"
  );
}
