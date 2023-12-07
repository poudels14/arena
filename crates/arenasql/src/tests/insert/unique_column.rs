use crate::tests::create_session_context;

#[tokio::test]
async fn dont_throw_when_inserting_unique_row() {
  let session = create_session_context();
  let txn = session.begin_transaction().unwrap();

  let _ = txn
    .execute_sql(
      r#"CREATE TABLE IF NOT EXISTS unique_column (
          id VARCHAR(50) UNIQUE,
          name TEXT
        )"#,
    )
    .await
    .unwrap();

  let result = txn
    .execute_sql(&format!(
      "INSERT INTO unique_column VALUES('random_id_1', 'name 1')"
    ))
    .await;

  assert!(result.is_ok());

  let result = txn
    .execute_sql(&format!(
      "INSERT INTO unique_column VALUES('random_id_2', 'name 1')"
    ))
    .await;

  assert!(result.is_ok())
}

#[tokio::test]
async fn throw_when_inserting_duplicate_row() {
  let session = create_session_context();
  let txn = session.begin_transaction().unwrap();

  let _ = txn
    .execute_sql(
      r#"CREATE TABLE IF NOT EXISTS unique_column (
          id VARCHAR(50) UNIQUE
          -- config DECIMAL(76, 1)
          -- config arrow_typeof(fixedsizebinary 10)
          -- config BYTEA
        )"#,
    )
    .await
    .unwrap();

  txn
    .execute_sql(&format!("SELECT * FROM unique_column;"))
    .await
    .unwrap();

  txn
    .execute_sql(&format!("INSERT INTO unique_column VALUES('random_id_1')"))
    .await
    .unwrap();

  let failed_query = txn
    .execute_sql(&format!("INSERT INTO unique_column VALUES('random_id_1')"))
    .await;

  assert!(failed_query.is_err())
}
