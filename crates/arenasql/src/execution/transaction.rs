use std::sync::Arc;

use datafusion::execution::context::{
  SQLOptions, SessionContext as DfSessionContext,
};
use datafusion::logical_expr::LogicalPlan;
use datafusion::physical_plan::{execute_stream, ExecutionPlan};
use sqlparser::ast::Statement as SQLStatement;

use super::response::ExecutionResponse;
use crate::df::plans;
use crate::{storage, Error, Result};

#[allow(unused)]
#[derive(Clone)]
pub struct Transaction {
  pub(crate) storage_txn: storage::Transaction,
  pub(super) sql_options: SQLOptions,
  pub(super) ctxt: DfSessionContext,
}

impl Transaction {
  pub async fn create_verified_logical_plan(
    &self,
    stmt: Box<SQLStatement>,
  ) -> Result<LogicalPlan> {
    let state = self.ctxt.state();
    let plan = state
      .statement_to_plan(datafusion::sql::parser::Statement::Statement(stmt))
      .await?;
    self.sql_options.verify_plan(&plan)?;
    Ok(plan)
  }

  pub async fn execute_sql(&self, sql: &str) -> Result<ExecutionResponse> {
    let mut stmts = crate::parser::parse_and_sanitize(sql)?;
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
    match plans::get_custom_execution_plan(&self.ctxt, &self.storage_txn, &stmt)
      .await?
    {
      Some(plan) => self.execute_stream(None, plan).await,
      None => {
        let state = self.ctxt.state();
        // TODO: creating physical plan from SQL is expensive
        // look into caching physical plans
        let plan = state
          .statement_to_plan(datafusion::sql::parser::Statement::Statement(
            stmt,
          ))
          .await?;

        self.sql_options.verify_plan(&plan)?;
        self.execute_logical_plan(plan).await
      }
    }
  }

  pub async fn execute_logical_plan(
    &self,
    plan: LogicalPlan,
  ) -> Result<ExecutionResponse> {
    let df = self.ctxt.execute_logical_plan(plan.clone()).await?;
    let physical_plan = df.create_physical_plan().await?;
    self.execute_stream(Some(plan), physical_plan).await
  }

  #[inline]
  pub fn closed(&self) -> bool {
    self.storage_txn.closed()
  }

  #[inline]
  pub fn commit(self) -> Result<()> {
    self.storage_txn.commit()
  }

  #[inline]
  pub fn rollback(self) -> Result<()> {
    self.storage_txn.rollback()
  }

  #[inline]
  async fn execute_stream(
    &self,
    logical_plan: Option<LogicalPlan>,
    physical_plan: Arc<dyn ExecutionPlan>,
  ) -> Result<ExecutionResponse> {
    let response =
      execute_stream(physical_plan.clone(), self.ctxt.task_ctx().into())?;
    ExecutionResponse::create(response, logical_plan).await
  }
}
