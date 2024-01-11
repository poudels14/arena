use crate::execute_query;
use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn delete_test_empty_table() {
  let session = create_session_context();
  let txn = session.new_active_transaction().unwrap();

  execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS test_table (
      id VARCHAR(50) UNIQUE,
      name TEXT
    )"#
  )
  .unwrap();

  let res = execute_query!(txn, r#"DELETE FROM test_table where id = 'id_1'"#);
  assert!(res.is_ok(), "Deleting rows from empty table");
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_test_delete_single_row() {
  let session = create_session_context();
  let txn = session.new_active_transaction().unwrap();

  execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS test_table (
      id VARCHAR(50),
      name TEXT
    )"#
  )
  .unwrap();

  execute_query!(
    txn,
    r#"INSERT INTO test_table(id, name)
      VALUES('id_1', 'name 1'),
      ('id_2', 'name'),
      ('id_3', 'name 3')"#
  )
  .unwrap();

  let _ =
    execute_query!(txn, r#"DELETE FROM test_table where id = 'id_1'"#).unwrap();

  let res = execute_query!(txn, r#"SELECT * FROM test_table"#).unwrap();
  assert_eq!(
    res.num_rows().await.unwrap(),
    2,
    "Select query expected to return 2 rows"
  );
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_test_delete_single_row_using_table_qualifier() {
  let session = create_session_context();
  let txn = session.new_active_transaction().unwrap();

  execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS test_table (
      id VARCHAR(50),
      name TEXT
    )"#
  )
  .unwrap();

  execute_query!(
    txn,
    r#"INSERT INTO test_table(id, name)
      VALUES('id_1', 'name 1'),
      ('id_2', 'name'),
      ('id_3', 'name 3')"#
  )
  .unwrap();

  // use test_table.id for selecting row
  let _ = execute_query!(
    txn,
    r#"DELETE FROM test_table where test_table.id = 'id_1'"#
  )
  .unwrap();

  let res = execute_query!(txn, r#"SELECT * FROM test_table"#).unwrap();
  assert_eq!(
    res.num_rows().await.unwrap(),
    2,
    "Select query expected to return 2 rows"
  );
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_test_delete_using_two_columns_filter() {
  let session = create_session_context();
  let txn = session.new_active_transaction().unwrap();

  execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS test_table (
      id VARCHAR(50),
      name TEXT
    )"#
  )
  .unwrap();

  execute_query!(
    txn,
    r#"INSERT INTO test_table(id, name)
      VALUES('id_1', 'name 1'),
      ('id_2', 'name'),
      ('id_3', 'name 3')"#
  )
  .unwrap();

  let _ = execute_query!(
    txn,
    r#"DELETE FROM test_table where id = 'id_1' AND name = 'name 1'"#
  )
  .unwrap();

  let res = execute_query!(txn, r#"SELECT * FROM test_table"#).unwrap();
  assert_eq!(
    res.num_rows().await.unwrap(),
    2,
    "Select query expected to return 2 rows"
  );
}
