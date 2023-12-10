use crate::execute_query;
use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn create_index_after_creating_table_in_same_transaction() {
  let session = create_session_context();

  let txn = session.begin_transaction().unwrap();
  let _ = execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS unique_column (
        id VARCHAR(50),
        name TEXT
      )"#
  );

  let res = execute_query!(
    txn,
    r#"CREATE INDEX unique_column_id_key ON public.unique_column(id, name)"#
  );

  assert!(res.is_ok());

  let res = execute_query!(txn, r#"SELECT * FROM unique_column;"#);
  assert!(res.is_ok());
  txn.commit().unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn create_index_after_creating_table_in_different_transaction() {
  let session = create_session_context();

  let txn = session.begin_transaction().unwrap();
  let res = execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS unique_column (
        id VARCHAR(50),
        name TEXT
      )"#
  );
  assert!(res.is_ok());

  txn.commit().unwrap();
  let txn = session.begin_transaction().unwrap();

  let res = execute_query!(
    txn,
    r#"CREATE INDEX unique_column_id_key ON public.unique_column(id, name)"#
  );
  assert!(res.is_ok());

  let txn = session.begin_transaction().unwrap();
  let res = execute_query!(txn, r#"SELECT * FROM unique_column;"#);
  assert!(res.is_ok());
}

#[tokio::test(flavor = "multi_thread")]
async fn create_index_with_same_name_in_same_txn_without_if_not_exist() {
  let session = create_session_context();

  let txn = session.begin_transaction().unwrap();
  let res = execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS unique_column (
        id VARCHAR(50),
        name TEXT
      )"#
  );
  assert!(res.is_ok());

  txn.commit().unwrap();
  let txn = session.begin_transaction().unwrap();

  let res = execute_query!(
    txn,
    r#"CREATE INDEX unique_column_id_key ON public.unique_column(id, name)"#
  );
  assert!(res.is_ok());

  let res = execute_query!(
    txn,
    r#"CREATE INDEX unique_column_id_key ON public.unique_column(id, name)"#
  );
  // This should throw error since the index with same name already exist
  assert!(res.is_err());
}
