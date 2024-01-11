use crate::execute_query;
use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn select_test_count_id() {
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
    r#"INSERT INTO test_table
      VALUES('random_id_1', 'name 1'),
      ('id_2', 'name'),
      ('id_3', 'name 3')"#
  )
  .unwrap();

  let res = execute_query!(txn, r#"SELECT count(id) FROM test_table"#).unwrap();
  assert_eq!(
    res.get_count().await.unwrap(),
    3,
    "Expected count(id) to be 3"
  )
}

#[tokio::test(flavor = "multi_thread")]
async fn select_test_column_alias() {
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

  let res =
    execute_query!(txn, r#"SELECT test_table.id as id1 FROM test_table"#)
      .unwrap();
  assert_eq!(
    res.num_rows().await.unwrap(),
    0,
    "Select query expected to succeed"
  );
}
