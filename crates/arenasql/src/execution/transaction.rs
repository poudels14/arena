use datafusion::execution::context::{
  SQLOptions, SessionContext as DfSessionContext,
};
use datafusion::physical_plan::execute_stream;
use sqlparser::ast::Statement as SQLStatement;

use super::response::ExecutionResponse;
use crate::{storage, Error, Result};

#[allow(unused)]
pub struct Transaction {
  pub(super) storage_txn: storage::Transaction,
  pub(super) sql_options: SQLOptions,
  pub(super) ctxt: DfSessionContext,
}

impl Transaction {
  pub async fn execute_sql(&self, sql: &str) -> Result<ExecutionResponse> {
    let mut stmts = crate::parser::parse(sql)?;
    if stmts.len() != 1 {
      return Err(Error::UnsupportedOperation(
        "In a transaction, one and only one statement should be executed"
          .to_owned(),
      ));
    }
    self.execute(stmts.pop().unwrap().into()).await
  }

  pub async fn execute(
    &self,
    stmt: Box<SQLStatement>,
  ) -> Result<ExecutionResponse> {
    let state = self.ctxt.state();

    // TODO: creating physical plan from SQL is expensive
    // look into caching physical plans
    let plan = state
      .statement_to_plan(datafusion::sql::parser::Statement::Statement(stmt))
      .await?;

    self.sql_options.verify_plan(&plan)?;
    let df = self.ctxt.execute_logical_plan(plan.clone()).await?;
    let physical_plan = df.create_physical_plan().await?;

    ExecutionResponse::create(
      plan,
      execute_stream(physical_plan, self.ctxt.task_ctx().into())?,
    )
    .await
  }

  pub fn commit(self) -> Result<()> {
    self.storage_txn.commit()
  }

  pub fn rollback(self) -> Result<()> {
    self.storage_txn.rollback()
  }
}
