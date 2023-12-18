use std::borrow::BorrowMut;
use std::sync::Arc;

use datafusion::execution::context::{
  SQLOptions, SessionContext as DfSessionContext,
};
use datafusion::logical_expr::LogicalPlan;
use datafusion::physical_plan::{execute_stream, ExecutionPlan};
use once_cell::sync::Lazy;
use sqlparser::ast::Statement as SQLStatement;

use super::response::ExecutionResponse;
use crate::ast::statement::StatementType;
use crate::df::plans::{create_index, insert_rows};
use crate::plans::{CustomExecutionPlanAdapter, ExecutionPlanExtension};
use crate::{storage, Error, Result};

pub const DEFAULT_EXTENSIONS: Lazy<Arc<Vec<ExecutionPlanExtension>>> =
  Lazy::new(|| Arc::new(vec![Arc::new(create_index::extension)]));

#[allow(unused)]
#[derive(Clone)]
pub struct Transaction {
  pub(crate) storage_txn: storage::Transaction,
  pub(super) sql_options: SQLOptions,
  pub(super) ctxt: DfSessionContext,
  pub(super) extensions: Arc<Vec<ExecutionPlanExtension>>,
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
    match StatementType::from(stmt.as_ref()).is_insert() {
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
    let stmt_type = StatementType::from(stmt.as_ref());
    let state = self.ctxt.state();
    let custom_plan = DEFAULT_EXTENSIONS
      .iter()
      .chain(self.extensions.iter())
      .find_map(|ext| ext(&state, &self.storage_txn, &stmt).transpose())
      .transpose()?;
    match custom_plan {
      Some(plan) => {
        self
          .execute_stream(
            &stmt_type,
            Arc::new(CustomExecutionPlanAdapter::new(plan)),
          )
          .await
      }
      None => {
        let logical_plan = self.create_verified_logical_plan(stmt).await?;
        self.execute_logical_plan(&stmt_type, logical_plan).await
      }
    }
  }

  #[inline]
  pub async fn execute_logical_plan(
    &self,
    stmt_type: &StatementType,
    plan: LogicalPlan,
  ) -> Result<ExecutionResponse> {
    let df = self.ctxt.execute_logical_plan(plan.clone()).await?;
    let physical_plan = df.create_physical_plan().await?;
    self.execute_stream(stmt_type, physical_plan).await
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
    stmt_type: &StatementType,
    physical_plan: Arc<dyn ExecutionPlan>,
  ) -> Result<ExecutionResponse> {
    let response =
      execute_stream(physical_plan.clone(), self.ctxt.task_ctx().into())?;
    ExecutionResponse::from_stream(stmt_type, response).await
  }
}
