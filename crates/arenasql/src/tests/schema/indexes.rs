use crate::execute_query;
use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn indexes_test_create_index_after_creating_table_in_same_transaction() {
  let session = create_session_context();

  let txn = session.new_active_transaction().unwrap();
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
async fn indexes_test_create_index_after_creating_table_in_diff_transaction() {
  let session = create_session_context();

  let txn = unsafe { session.get_or_create_active_transaction() };
  let res = execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS unique_column (
        id VARCHAR(50),
        name TEXT
      )"#
  );
  assert!(res.is_ok());
  txn.commit().unwrap();

  let txn = session.new_active_transaction().unwrap();
  let res = execute_query!(
    txn,
    r#"CREATE INDEX unique_column_id_key ON public.unique_column(id, name)"#
  );
  assert!(res.is_ok());

  let txn = session.new_active_transaction().unwrap();
  let res = execute_query!(txn, r#"SELECT * FROM unique_column;"#);
  assert!(res.is_ok());
}

#[tokio::test(flavor = "multi_thread")]
async fn indexes_test_index_with_same_name_in_same_txn_without_if_not_exist() {
  let session = create_session_context();

  let txn = session.new_active_transaction().unwrap();
  let res = execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS unique_column (
        id VARCHAR(50),
        name TEXT
      )"#
  );
  assert!(res.is_ok());

  txn.commit().unwrap();
  let txn = session.new_active_transaction().unwrap();

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

#[tokio::test(flavor = "multi_thread")]
/// This will test whether the new index that's created after
/// the table already has rows will be backfilled properly
async fn indexes_test_create_unique_index_and_verify_index_backfill() {
  let session = create_session_context();

  let txn = unsafe { session.get_or_create_active_transaction() };
  let _ = execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS test_table (
        id VARCHAR(50),
        name TEXT
      )"#
  );

  let res =
    execute_query!(txn, r#"INSERT INTO test_table VALUES('id1', 'name 1')"#);
  assert!(res.is_ok());

  // Create index after inserting some rows first
  let res = execute_query!(
    txn,
    r#"CREATE UNIQUE INDEX test_table_id_key ON test_table(id)"#
  );
  assert!(res.is_ok());

  // #####################################################################
  // Selecting columns that are not present in the index does table scan
  // instead of index scan. So, this will return all rows in the table
  // regardless of whether the index was populated
  let res = execute_query!(txn, r#"SELECT id, name FROM test_table"#).unwrap();
  assert_eq!(
    res.num_rows().await.unwrap(),
    1,
    "All the rows weren't returned even when using table scan"
  );

  // #####################################################################
  // Selecting columns that are present in the index only runs index scan
  // So, this should return same number of rows as present in the table
  let res = execute_query!(txn, r#"SELECT id FROM test_table"#).unwrap();
  assert_eq!(res.num_rows().await.unwrap(), 1, "Index wasn't backfilled");

  // #####################################################################
  // This should THROW error because of duplicate entry
  let res =
    execute_query!(txn, r#"INSERT INTO test_table VALUES('id1', 'name 1')"#);
  assert!(res.is_err());

  // #####################################################################
  // Insert non-duplicate row
  let res =
    execute_query!(txn, r#"INSERT INTO test_table VALUES('id_2', 'name 2')"#);
  assert!(res.is_ok());

  // #####################################################################
  // Should return rows inserted before and after creating index
  let res = execute_query!(txn, r#"SELECT id FROM test_table"#).unwrap();
  assert_eq!(
    res.num_rows().await.unwrap(),
    2,
    "Didn't add row to the new index"
  );

  // #####################################################################
  // Should return rows inserted before and after creating index
  let res = execute_query!(txn, r#"SELECT id, name FROM test_table"#).unwrap();
  assert_eq!(
    res.num_rows().await.unwrap(),
    2,
    "New row added after index isn't returned"
  );

  txn.commit().unwrap();
}

#[tokio::test(flavor = "multi_thread")]
/// This will test whether the new index that's created after
/// the table already has rows will be backfilled properly
async fn indexes_test_create_secondary_index_and_verify_index_backfill() {
  let session = create_session_context();

  let txn = session.new_active_transaction().unwrap();
  let _ = execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS test_table (
        id VARCHAR(50),
        name TEXT
      )"#
  );

  let res =
    execute_query!(txn, r#"INSERT INTO test_table VALUES('id1', 'name 1')"#);
  assert!(res.is_ok());

  // Create index after inserting some rows first
  let res =
    execute_query!(txn, r#"CREATE INDEX test_table_id_key ON test_table(id)"#);
  assert!(res.is_ok());

  // #####################################################################
  // Selecting columns that are not present in the index does table scan
  // instead of index scan. So, this will return all rows in the table
  // regardless of whether the index was populated
  let res = execute_query!(txn, r#"SELECT id, name FROM test_table"#).unwrap();
  assert_eq!(
    res.num_rows().await.unwrap(),
    1,
    "All the rows weren't returned even when using table scan"
  );

  // #####################################################################
  // Selecting columns that are present in the index only runs index scan
  // So, this should return same number of rows as present in the table
  let res = execute_query!(txn, r#"SELECT id FROM test_table"#).unwrap();
  assert_eq!(res.num_rows().await.unwrap(), 1, "Index wasn't backfilled");

  // #####################################################################
  // Duplicate entry doesn't throw error because it's not a unique index
  let res =
    execute_query!(txn, r#"INSERT INTO test_table VALUES('id1', 'name 1')"#);
  assert!(res.is_ok());

  // #####################################################################
  // Insert non-duplicate row
  let res =
    execute_query!(txn, r#"INSERT INTO test_table VALUES('id_2', 'name 2')"#);
  assert!(res.is_ok());

  // #####################################################################
  // Should return rows inserted before and after creating index
  let res = execute_query!(txn, r#"SELECT id FROM test_table"#).unwrap();
  assert_eq!(
    res.num_rows().await.unwrap(),
    3,
    "Didn't add row to the new index"
  );

  // #####################################################################
  // Should return rows inserted before and after creating index
  let res = execute_query!(txn, r#"SELECT id, name FROM test_table"#).unwrap();
  assert_eq!(
    res.num_rows().await.unwrap(),
    3,
    "New row added after index isn't returned"
  );

  txn.commit().unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn indexes_test_create_unique_index_with_multiple_cols() {
  let session = create_session_context();

  let txn = session.new_active_transaction().unwrap();
  let _ = execute_query!(
    txn,
    r#"CREATE TABLE IF NOT EXISTS test_table (
        id VARCHAR(50),
        name TEXT
      )"#
  );

  let res =
    execute_query!(txn, r#"INSERT INTO test_table VALUES('id1', 'name 1')"#);
  assert!(res.is_ok());

  // Create index after inserting some rows first
  let res = execute_query!(
    txn,
    r#"CREATE UNIQUE INDEX test_table_index ON test_table(id, name)"#
  );
  assert!(res.is_ok());

  // #####################################################################
  // Selecting columns that are present in the index only runs index scan
  // So, this should return same number of rows as present in the table
  let res = execute_query!(txn, r#"SELECT id FROM test_table"#).unwrap();
  // let res: Vec<RecordBatch> = res.unwrap().try_collect().await.unwrap();
  assert_eq!(res.num_rows().await.unwrap(), 1, "Index wasn't backfilled");

  // #####################################################################
  // Duplicate entry SHOULD throw error because it's an unique index
  let res =
    execute_query!(txn, r#"INSERT INTO test_table VALUES('id1', 'name 1')"#);
  assert!(res.is_err());

  // #####################################################################
  // Insert non-duplicate row
  let res =
    execute_query!(txn, r#"INSERT INTO test_table VALUES('id1', 'name 2')"#);
  assert!(res.is_ok());

  txn.commit().unwrap();
}
