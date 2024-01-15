use datafusion::arrow::array::as_string_array;

use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn vector_test_l2_distance() {
  let session = create_session_context();

  session
    .execute_sql(
      r#"CREATE TABLE IF NOT EXISTS vectors (
      id VARCHAR(50),
      embeddings VECTOR(4)
    )"#,
    )
    .await
    .unwrap();

  session
    .execute_sql(&format!(
      "INSERT INTO vectors VALUES ('id1', [0.1, 0.2, 0.3,0.4])"
    ))
    .await
    .unwrap();
  session
    .execute_sql(&format!(
      "INSERT INTO vectors VALUES ('id2', [1.4, 1.3, 1.2, 1.1])"
    ))
    .await
    .unwrap();

  let mut res = session
  .execute_sql(
    r#"SELECT id as dist FROM vectors WHERE l2(embeddings, [1.0, 1.0, 1.0, 1.0]) < 1.1"#
  )
  .await
  .unwrap();

  let batch = res
    .pop()
    .unwrap()
    .collect_batches()
    .await
    .unwrap()
    .pop()
    .unwrap();

  assert_eq!(batch.num_rows(), 1, "Expected count(id) to be 1");
  let id = as_string_array(batch.column(0)).value(0);
  assert_eq!(id, "id1");
}

#[tokio::test(flavor = "multi_thread")]
async fn vector_test_create_table_with_odd_length_vector_column() {
  let session = create_session_context();

  let res = session
    .execute_sql(
      r#"CREATE TABLE IF NOT EXISTS vectors (
      id VARCHAR(50),
      embeddings VECTOR(3)
    )"#,
    )
    .await;
  assert!(res.is_err());
}
