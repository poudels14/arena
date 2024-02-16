use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn table_schema_alter_table_add_column_on_empty_table() {
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

  let res = session
    .execute_sql(r#"ALTER TABLE test_table ADD COLUMN description TEXT"#)
    .await;
  assert!(res.is_ok());

  // adding new row without description should error
  let res = session
    .execute_sql(r#"INSERT INTO test_table VALUES('id1', 'name 1')"#)
    .await;
  assert!(res.is_err());

  // inserting with description should succeed
  let res = session
    .execute_sql(
      r#"INSERT INTO test_table VALUES('id2', 'name 2', 'description 2')"#,
    )
    .await;
  assert!(res.is_ok());

  let res = session
    .execute_sql(r#"SELECT id FROM test_table"#)
    .await
    .unwrap()
    .pop()
    .unwrap();
  assert_eq!(res.num_rows().await.unwrap(), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn table_schema_alter_table_add_column_on_non_empty_table() {
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

  // add a row before adding description column
  let res = session
    .execute_sql(r#"INSERT INTO test_table VALUES('id1', 'name 1')"#)
    .await;
  assert!(res.is_ok());

  let res = session
    .execute_sql(r#"ALTER TABLE test_table ADD COLUMN description TEXT"#)
    .await;
  assert!(res.is_ok());

  // inserting with description should succeed
  let res = session
    .execute_sql(
      r#"INSERT INTO test_table VALUES('id2', 'name 2', 'description 2')"#,
    )
    .await;
  assert!(res.is_ok());

  let res = session
    .execute_sql(r#"SELECT id FROM test_table"#)
    .await
    .unwrap()
    .pop()
    .unwrap();
  assert_eq!(res.num_rows().await.unwrap(), 2);
}

#[tokio::test(flavor = "multi_thread")]
async fn table_schema_alter_table_add_timestamp_column_on_empty_table() {
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

  let res = session
    .execute_sql(r#"ALTER TABLE test_table ADD COLUMN description TIMESTAMP"#)
    .await;
  assert!(res.is_ok());
}
