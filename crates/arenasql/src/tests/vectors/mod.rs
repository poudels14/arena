use std::sync::Arc;

use datafusion::arrow::array::{as_string_array, ListArray};
use datafusion::arrow::datatypes::Float32Type;
use datafusion::scalar::ScalarValue;

mod index;

use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn vector_column_type_test_l2_distance() {
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
    r#"SELECT id as dist FROM vectors WHERE l2(embeddings, '[1.0, 1.0, 1.0, 1.0]') < 1.1"#
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
async fn vector_column_type_test_l2_distance_sorting() {
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
      "INSERT INTO vectors VALUES
      ('id1', [0.1, 0.2, 0.3,0.4]),
      ('id2', [1.4, 1.3, 1.2, 1.1])"
    ))
    .await
    .unwrap();

  let mut res = session
    .execute_sql(
      r#"SELECT id as dist
        FROM vectors
        ORDER BY l2(embeddings, '[1.0, 1.0, 1.0, 1.0]')
        LIMIT 1"#,
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
async fn vector_column_type_test_create_table_with_odd_length_vector_column() {
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

#[tokio::test(flavor = "multi_thread")]
async fn vector_column_type_test_params() {
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

  let stmts =
    crate::ast::parse("INSERT INTO vectors VALUES ('id1', $1)").unwrap();
  session
    .execute_statement(
      stmts[0].clone().into(),
      None,
      Some(vec![ScalarValue::List(Arc::new(
        ListArray::from_iter_primitive::<Float32Type, _, _>(vec![Some(vec![
          Some(0.1f32),
          Some(0.2),
          Some(0.3),
          Some(0.4),
        ])]),
      ))]),
    )
    .await
    .unwrap();

  let mut res = session
    .execute_sql(r#"SELECT id FROM vectors"#)
    .await
    .unwrap();

  assert_eq!(
    res.pop().unwrap().num_rows().await.unwrap(),
    1,
    "Expected number of rows to be 1"
  );
}
