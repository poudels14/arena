use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn transaction_autocommit_unchained_transaction() {
  let session = create_session_context();

  let _ = session
    .execute_sql(
      r#"CREATE TABLE IF NOT EXISTS test_table (
        id VARCHAR(50),
        name TEXT
      )"#,
    )
    .await
    .unwrap();

  // Use new transaction to query the table and if the table exists,
  // it means previous transaction was committed
  let txn = session.new_transaction().unwrap();
  let res = txn.execute_sql("SELECT * FROM test_table;").await.unwrap();
  // Make sure SELECT is successful
  assert_eq!(res.num_rows().await.unwrap(), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn transaction_autorollback_uncommitted_chained_transaction() {
  let session = create_session_context();

  let _ = session.execute_sql(r#"BEGIN"#).await.unwrap();
  let _ = session
    .execute_sql(
      r#"CREATE TABLE IF NOT EXISTS test_table (
        id VARCHAR(50),
        name TEXT
      )"#,
    )
    .await
    .unwrap();

  // Use new transaction to query the table and if the table exists,
  // it means previous transaction was committed
  let txn = session.new_transaction().unwrap();
  let res = txn.execute_sql("SELECT * FROM test_table;").await;

  assert!(
    res.is_err(),
    "New transaction shouldn't find uncommitted transaction's data"
  );
}

#[tokio::test(flavor = "multi_thread")]
async fn transaction_autorollback_uncommitted_chained_transaction_session() {
  let session = create_session_context();

  let _ = session.execute_sql(r#"BEGIN"#).await.unwrap();
  let _ = session
    .execute_sql(
      r#"CREATE TABLE IF NOT EXISTS test_table (
        id VARCHAR(50),
        name TEXT
      )"#,
    )
    .await
    .unwrap();

  drop(session);

  let session2 = create_session_context();

  // Use new session to query the table and if the table exists,
  // it means previous transaction was committed
  let res = session2.execute_sql("SELECT * FROM test_table;").await;

  assert!(
    res.is_err(),
    "New transaction shouldn't find uncommitted transaction's data"
  );
}

#[tokio::test(flavor = "multi_thread")]
async fn transaction_commit_statement_should_commit_chained_transaction() {
  let session = create_session_context();

  let _ = session.execute_sql(r#"BEGIN"#).await.unwrap();
  let _ = session
    .execute_sql(
      r#"CREATE TABLE IF NOT EXISTS test_table (
        id VARCHAR(50),
        name TEXT
      )"#,
    )
    .await
    .unwrap();
  let _ = session.execute_sql(r#"COMMIT"#).await.unwrap();

  // Use new transaction to query the table and if the table exists,
  // it means previous transaction was committed
  let txn = session.new_transaction().unwrap();
  let res = txn.execute_sql("SELECT * FROM test_table;").await.unwrap();
  // Make sure SELECT is successful
  assert_eq!(res.num_rows().await.unwrap(), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn transaction_create_index_after_creating_table_in_same_txn() {
  let session = create_session_context();

  let _ = session.execute_sql(r#"BEGIN"#).await.unwrap();
  let _ = session
    .execute_sql(
      r#"CREATE TABLE IF NOT EXISTS test_table (
        id VARCHAR(50),
        name TEXT
      )"#,
    )
    .await
    .unwrap();

  let res = session
    .execute_sql(
      "CREATE INDEX IF NOT EXISTS test_table_index ON test_table(id);",
    )
    .await;
  assert!(res.is_ok());
}

#[tokio::test(flavor = "multi_thread")]
async fn transaction_create_index_after_committing_a_transaction() {
  let session = create_session_context();

  let _ = session.execute_sql(r#"BEGIN"#).await.unwrap();
  let _ = session
    .execute_sql(
      r#"CREATE TABLE IF NOT EXISTS test_table (
        id VARCHAR(50),
        name TEXT
      )"#,
    )
    .await
    .unwrap();
  let _ = session.execute_sql(r#"COMMIT"#).await.unwrap();

  let res = session
    .execute_sql(
      "CREATE INDEX IF NOT EXISTS test_table_index ON test_table(id);",
    )
    .await;
  assert!(res.is_ok());
}
