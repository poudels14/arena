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

#[tokio::test(flavor = "multi_thread")]
async fn select_test_with_index_using_or_in_unique_column() {
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

  execute_query!(
    txn,
    r#"INSERT INTO test_table
      VALUES('id_1', 'name 1'),
      ('id_2', 'name'),
      ('id_3', 'name 3')"#
  )
  .unwrap();

  let res = execute_query!(
    txn,
    r#"SELECT id FROM test_table WHERE id = 'id_1' OR id = 'id_2'"#
  )
  .unwrap();
  assert_eq!(
    res.num_rows().await.unwrap(),
    2,
    "Select query expected to succeed"
  );
}

#[tokio::test(flavor = "multi_thread")]
async fn select_test_with_index_using_or_in_secondary_column() {
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

  execute_query!(txn, r#"CREATE INDEX test_index ON test_table (id);"#)
    .unwrap();

  execute_query!(
    txn,
    r#"INSERT INTO test_table
      VALUES('id_1', 'name 1'),
      ('id_2', 'name'),
      ('id_3', 'name 3')"#
  )
  .unwrap();

  let res = execute_query!(
    txn,
    r#"SELECT id FROM test_table WHERE id = 'id_1' OR name = 'name'"#
  )
  .unwrap();
  assert_eq!(
    res.num_rows().await.unwrap(),
    2,
    "Select query expected to succeed"
  );
}

#[tokio::test(flavor = "multi_thread")]
async fn select_test_with_multiple_indexes() {
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

  execute_query!(txn, r#"CREATE INDEX test_index_name ON test_table (name);"#)
    .unwrap();

  execute_query!(
    txn,
    r#"INSERT INTO test_table
      VALUES('id_1', 'name 1'),
      ('id_2', 'name'),
      ('id_3', 'name 3')"#
  )
  .unwrap();

  let res = execute_query!(
    txn,
    r#"SELECT id FROM test_table WHERE id = 'id_1' OR name = 'name'"#
  )
  .unwrap();
  assert_eq!(
    res.num_rows().await.unwrap(),
    2,
    "Select query expected to succeed"
  );
}

#[tokio::test(flavor = "multi_thread")]
async fn select_test_with_multiple_indexes_and_one_returns_nothing() {
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

  execute_query!(txn, r#"CREATE INDEX test_index_name ON test_table (name);"#)
    .unwrap();

  execute_query!(
    txn,
    r#"INSERT INTO test_table
      VALUES('id_1', 'name 1'),
      ('id_2', 'name'),
      ('id_3', 'name 3')"#
  )
  .unwrap();

  let res = execute_query!(
    txn,
    r#"SELECT id FROM test_table WHERE id = 'id_1' OR name = 'name69'"#
  )
  .unwrap();
  assert_eq!(
    res.num_rows().await.unwrap(),
    1,
    "Select query expected to succeed"
  );
}
