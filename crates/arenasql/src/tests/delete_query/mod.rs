use crate::execute_query;
use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn delete_test_delete_single_row() {
  let session = create_session_context();
  let txn = session.begin_transaction().unwrap();

  execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS test_table (
      id VARCHAR(50),
      name TEXT
      --,
      --age INTEGER
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
