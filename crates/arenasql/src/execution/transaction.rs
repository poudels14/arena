use std::borrow::BorrowMut;
use std::sync::Arc;

use datafusion::execution::context::{
  SQLOptions, SessionContext as DfSessionContext,
};
use datafusion::logical_expr::LogicalPlan;
use datafusion::physical_plan::{execute_stream, ExecutionPlan};
use sqlparser::ast::Statement as SQLStatement;

use super::response::ExecutionResponse;
use crate::ast::statement::StatementType;
use crate::df::plans::{self, insert_rows};
use crate::{storage, Error, Result};

#[allow(unused)]
#[derive(Clone)]
pub struct Transaction {
  pub(crate) storage_txn: storage::Transaction,
  pub(super) sql_options: SQLOptions,
  pub(super) ctxt: DfSessionContext,
}

impl Transaction {
  #[inline]
  pub async fn create_verified_logical_plan(
    &self,
    mut stmt: Box<SQLStatement>,
  ) -> Result<LogicalPlan> {
    let state = self.ctxt.state();

    // Modify stmt if needed
    // THIS IS A HACK needed because table scan needs to return rowid
    // for delete/update
    match stmt.as_ref().is_insert() {
      true => {
        let stmt: &mut SQLStatement = stmt.borrow_mut();
        insert_rows::set_explicit_columns_in_insert_query(&state, stmt).await?;
      }
      _ => {}
    };

    // TODO: creating physical plan from SQL is expensive
    // look into caching physical plans
    let plan = state
      .statement_to_plan(datafusion::sql::parser::Statement::Statement(stmt))
      .await?;
    self.sql_options.verify_plan(&plan)?;
    Ok(plan)
  }

  #[inline]
  pub async fn execute_sql(&self, sql: &str) -> Result<ExecutionResponse> {
    let mut stmts = crate::ast::parse_and_sanitize(sql)?;
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
    match plans::get_custom_execution_plan(&state, &self.storage_txn, &stmt)
      .await?
    {
      Some(plan) => self.execute_stream(None, plan).await,
      None => {
        let logical_plan = self.create_verified_logical_plan(stmt).await?;
        self.execute_logical_plan(logical_plan).await
      }
    }
  }

  #[inline]
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
    ExecutionResponse::from_stream_and_plan(response, logical_plan).await
  }
}
