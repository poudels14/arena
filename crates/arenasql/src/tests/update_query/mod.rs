use crate::execute_query;
use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn update_test_update_single_column_and_single_row() {
  let session = create_session_context();
  let txn = session.new_active_transaction().unwrap();

  execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS test_table (
      id VARCHAR(50),
      name TEXT,
      age INTEGER
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

  let res =
    execute_query!(txn, r#"UPDATE test_table SET age = 11 where id = 'id_1'"#);

  assert!(res.is_ok());

  let res =
    execute_query!(txn, r#"SELECT count(*) FROM test_table WHERE age = 11"#)
      .unwrap();
  assert_eq!(
    res.get_count().await.unwrap(),
    1,
    "Select returned more than one row"
  );
}

#[tokio::test(flavor = "multi_thread")]
async fn update_test_update_with_filter_selecting_multiple_rows() {
  let session = create_session_context();
  let txn = session.new_active_transaction().unwrap();

  execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS test_table (
      id VARCHAR(50),
      name TEXT,
      age INTEGER
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

  let res =
    execute_query!(txn, r#"UPDATE test_table SET age = 11 where id > 'id_1'"#);

  assert!(res.is_ok());

  let res =
    execute_query!(txn, r#"SELECT count(*) FROM test_table WHERE age = 11"#)
      .unwrap();

  assert_eq!(
    res.get_count().await.unwrap(),
    2,
    "Select returned more than one row"
  );
}

#[tokio::test(flavor = "multi_thread")]
async fn update_test_update_multiple_columns_and_multiple_rows() {
  let session = create_session_context();
  let txn = session.new_active_transaction().unwrap();

  execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS test_table (
      id VARCHAR(50),
      name TEXT,
      age INTEGER
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

  let res = execute_query!(
    txn,
    r#"SELECT count(*) FROM test_table WHERE age = 11 and name = 'new_name'"#
  )
  .unwrap();
  assert_eq!(
    res.get_count().await.unwrap(),
    0,
    "Select returned more than one row"
  );

  execute_query!(
    txn,
    r#"UPDATE test_table SET age = 11, name = 'new_name' where id > 'id_1'"#
  )
  .unwrap();

  let res = execute_query!(
    txn,
    r#"SELECT count(*) FROM test_table WHERE age = 11 and name = 'new_name'"#
  )
  .unwrap();
  assert_eq!(
    res.get_count().await.unwrap(),
    2,
    "Select returned more than one row"
  );
}
