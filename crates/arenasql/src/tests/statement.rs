use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn statement_test_begin_transaction() {
  let session = create_session_context();
  let txn = session.begin_transaction().unwrap();

  let stmt = Box::new(crate::ast::parse("BEGIN").unwrap()[0].clone());
  let plan = txn.create_verified_logical_plan(stmt).await;
  assert!(plan.is_ok());
}
