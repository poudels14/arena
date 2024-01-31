use crate::execute_query;
use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn crate_table_with_file_column() {
  let session = create_session_context();
  let txn = session.new_active_transaction().unwrap();

  let res = execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS files_table (
      data FILE
    )"#
  );

  assert!(res.is_ok(), "Failed to create TABLE with FILE column");

  execute_query!(
    txn,
    r#"INSERT INTO files_table
      VALUES('{{"content": "arena"}}'),
      ('{{"endpoint" : "endpoint","bucket": "bucket1", "path": "p1"}}')"#
  )
  .unwrap();

  let res = execute_query!(txn, r#"SELECT * FROM files_table"#).unwrap();
  assert_eq!(
    res.num_rows().await.unwrap(),
    2,
    "Number of rows didn't match"
  );
}
