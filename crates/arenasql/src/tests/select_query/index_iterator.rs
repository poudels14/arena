use datafusion::common::Column;
use datafusion::logical_expr::Expr;
use datafusion::scalar::ScalarValue;

use crate::execute_query;
use crate::execution::filter::Filter;
use crate::execution::iterators::IndexIterator;
use crate::execution::DEFAULT_SCHEMA_NAME;
use crate::schema::{DataFrame, DataType};
use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn eq_filter_returns_single_row_iterator() {
  let session = create_session_context();
  let txn = session.new_active_transaction().unwrap();

  execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS unique_column (
      id VARCHAR(50) UNIQUE,
      name TEXT
    )"#
  )
  .unwrap();

  execute_query!(
    txn,
    r#"INSERT INTO unique_column
      VALUES('random_id_1', 'name 1'),
      ('id_2', 'name'),
      ('id_3', 'name 3')"#
  )
  .unwrap();

  let storage = txn.handle().lock(false).unwrap();
  let table = storage
    .get_table_schema(
      &session.config.catalog,
      DEFAULT_SCHEMA_NAME,
      "unique_column",
    )
    .unwrap()
    .unwrap();

  let id_index = table.indexes.get(0).unwrap();

  let id_eq_expr = Expr::Column(Column::new_unqualified("id"))
    .eq(Expr::Literal(ScalarValue::Utf8(Some("id_2".to_owned()))));
  let filters = vec![Filter::for_table(&table, &id_eq_expr).unwrap()];

  let column_projection = vec![0];
  let mut dataframe =
    DataFrame::with_capacity(100, vec![("id".to_owned(), DataType::Text)]);
  let index_iterator = IndexIterator::new(
    &storage,
    &table,
    id_index,
    &filters,
    &column_projection,
  );

  index_iterator.fill_into(&mut dataframe).unwrap();
  assert_eq!(dataframe.row_count(), 1)
}

#[tokio::test(flavor = "multi_thread")]
async fn le_filter_returns_multi_row_iterator() {
  let session = create_session_context();
  let txn = session.new_active_transaction().unwrap();

  execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS unique_column (
      id VARCHAR(50) UNIQUE,
      name TEXT
    )"#
  )
  .unwrap();

  execute_query!(
    txn,
    r#"INSERT INTO unique_column
      VALUES
      ('id_1', 'name 1'),
      ('id_2', 'name'),
      ('id_3', 'name 3')"#
  )
  .unwrap();

  let storage = txn.handle().lock(false).unwrap();
  let table = storage
    .get_table_schema(
      &session.config.catalog,
      DEFAULT_SCHEMA_NAME,
      "unique_column",
    )
    .unwrap()
    .unwrap();

  let id_eq_expr = Expr::Column(Column::new_unqualified("id"))
    .lt_eq(Expr::Literal(ScalarValue::Utf8(Some("id_2".to_owned()))));

  let filters = vec![Filter::for_table(&table, &id_eq_expr).unwrap()];

  let id_index = table.indexes.get(0).unwrap();
  let column_projection = vec![0];
  let mut dataframe =
    DataFrame::with_capacity(10, vec![("text".to_owned(), DataType::Text)]);
  let rows_iterator = IndexIterator::new(
    &storage,
    &table,
    id_index,
    &filters,
    &column_projection,
  );

  rows_iterator.fill_into(&mut dataframe).unwrap();

  // Since only '=' filter is applied during scanning right now,
  // other operator will return all rows
  assert_eq!(dataframe.row_count(), 3)
}
