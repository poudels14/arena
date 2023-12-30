use crate::execute_query;
use crate::storage::KeyValueGroup;
use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn drop_table_test_all_data_deleted() {
  let session = create_session_context();
  let txn = session.new_transaction().unwrap();

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
    r#"CREATE UNIQUE INDEX test_table_id ON test_table(id)"#
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

  let _ = execute_query!(txn, r#"DROP TABLE test_table"#).unwrap();

  let handle = txn.handle();
  let storage = handle.lock(true).unwrap();

  let index_scanner = storage
    .kv
    .scan_with_prefix(KeyValueGroup::IndexRows, &vec![])
    .unwrap();
  assert_eq!(index_scanner.get(), None, "Expected index rows to be empty");

  let index_scanner = storage
    .kv
    .scan_with_prefix(KeyValueGroup::Schemas, &vec![])
    .unwrap();
  assert_eq!(index_scanner.get(), None, "Expected schemas to be empty");

  let index_scanner = storage
    .kv
    .scan_with_prefix(KeyValueGroup::Rows, &vec![])
    .unwrap();
  assert_eq!(index_scanner.get(), None, "Expected table rows to be empty");
}
