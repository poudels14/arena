use crate::execute_query;
use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn crate_table_with_json_column() {
  let session = create_session_context();
  let txn = session.new_active_transaction().unwrap();

  let res = execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS json_table (
      data JSONB
    )"#
  );

  assert!(res.is_ok(), "Failed to create TABLE with JSONB column");

  execute_query!(
    txn,
    r#"INSERT INTO json_table
      VALUES('{{"name": "arena", "year": 2023}}'),
      ('{{"name": "arena-2", "year": 2024}}')"#
  )
  .unwrap();

  let res = execute_query!(txn, r#"SELECT * FROM json_table"#).unwrap();
  assert_eq!(
    res.num_rows().await.unwrap(),
    2,
    "Number of rows didn't match"
  );
}
