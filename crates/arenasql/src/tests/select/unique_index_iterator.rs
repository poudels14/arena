use datafusion::common::Column;
use datafusion::logical_expr::Expr;
use datafusion::scalar::ScalarValue;

use crate::df::scan::filter::Filter;
use crate::df::scan::unique_index_iterator;
use crate::execute_query;
use crate::storage::RowIterator;
use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn eq_filter_returns_single_row_iterator() {
  let session = create_session_context();
  let txn = session.begin_transaction().unwrap();

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

  let storage = txn.storage_txn.lock().unwrap();
  let table = storage
    .get_table_schema(
      &session.config.catalog,
      &session.config.default_schema,
      "unique_column",
    )
    .unwrap()
    .unwrap();

  let id_index = table.indexes.get(0).unwrap();

  let id_eq_expr = Expr::Column(Column::new_unqualified("id"))
    .eq(Expr::Literal(ScalarValue::Utf8(Some("id_2".to_owned()))));
  let filters = vec![Filter::for_table(&table, &id_eq_expr).unwrap()];

  let mut rows_iterator =
    unique_index_iterator::new(&table, id_index, &filters, &storage).unwrap()
      as Box<dyn RowIterator>;

  let mut count = 0;
  while let Some((_key, _)) = rows_iterator.get() {
    count += 1;
    rows_iterator.next()
  }

  assert_eq!(count, 1)
}

#[tokio::test(flavor = "multi_thread")]
async fn le_filter_returns_multi_row_iterator() {
  let session = create_session_context();
  let txn = session.begin_transaction().unwrap();

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

  let storage = txn.storage_txn.lock().unwrap();
  let table = storage
    .get_table_schema(
      &session.config.catalog,
      &session.config.default_schema,
      "unique_column",
    )
    .unwrap()
    .unwrap();

  let id_index = table.indexes.get(0).unwrap();

  let id_eq_expr = Expr::Column(Column::new_unqualified("id"))
    .lt_eq(Expr::Literal(ScalarValue::Utf8(Some("id_2".to_owned()))));

  let filters = vec![Filter::for_table(&table, &id_eq_expr).unwrap()];

  let mut rows_iterator =
    unique_index_iterator::new(&table, id_index, &filters, &storage).unwrap()
      as Box<dyn RowIterator>;

  let mut count = 0;
  while let Some(_) = rows_iterator.get() {
    count += 1;
    rows_iterator.next()
  }

  // Since only '=' filter is applied during scanning right now,
  // other operator will return all rows
  assert_eq!(count, 3)
}
