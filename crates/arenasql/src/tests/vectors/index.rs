use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn vector_create_hnsw_index() {
  let session = create_session_context();

  session
    .execute_sql(
      r#"CREATE TABLE IF NOT EXISTS vectors (
      id VARCHAR(50),
      parent_id VARCHAR(50),
      embeddings VECTOR(4)
    )"#,
    )
    .await
    .unwrap();

  session
    .execute_sql(
      r#"CREATE INDEX vectors_index ON vectors
      USING hnsw (embeddings)
      WITH (
        metric = 'l2',
        namespace = 'parent_id',
        M = 2,
        ef_construction = 10,
        ef = 4,
        dim = 3
      ) 
      
      "#,
    )
    .await
    .unwrap();

  let res = session
    .execute_sql(
      "SELECT * FROM vectors WHERE l2(embeddings, '[1.0, 1.0, 1.0, 1.0]') > 10;",
    )
    .await;
  assert!(res.is_ok());
}
