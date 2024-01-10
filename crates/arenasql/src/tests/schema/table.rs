use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn table_schema_shouldnt_be_found_after_its_dropped() {
  let session = create_session_context();

  let _ = session
    .execute_sql(
      r#"CREATE TABLE IF NOT EXISTS unique_column (
        id VARCHAR(50),
        name TEXT
      )"#,
    )
    .await
    .unwrap();

  let res = session.execute_sql(r#"DROP TABLE unique_column"#).await;
  assert!(res.is_ok());

  let res = session.execute_sql(r#"SELECT * FROM unique_column;"#).await;
  assert!(res.is_err());
}
