use std::borrow::BorrowMut;
use std::sync::Arc;

use datafusion::execution::context::{
  SQLOptions, SessionContext as DfSessionContext,
};
use datafusion::logical_expr::LogicalPlan;
use datafusion::physical_plan::{execute_stream, ExecutionPlan};
use derive_new::new;
use getset::Getters;
use once_cell::sync::Lazy;
use sqlparser::ast::Statement as SQLStatement;

use super::response::ExecutionResponse;
use super::{CustomExecutionPlanAdapter, ExecutionPlanExtension};
use super::{SessionConfig, SessionState};
use crate::ast::statement::StatementType;
use crate::df::plans::{create_index, insert_rows};
use crate::{storage, Error, Result};

pub const DEFAULT_EXTENSIONS: Lazy<Arc<Vec<ExecutionPlanExtension>>> =
  Lazy::new(|| Arc::new(vec![Arc::new(create_index::extension)]));

#[derive(Clone, Getters, new)]
pub struct Transaction {
  #[getset(get = "pub")]
  session_config: Arc<SessionConfig>,
  session_state: Arc<SessionState>,
  #[getset(get = "pub")]
  storage_transaction: storage::Transaction,
  sql_options: SQLOptions,
  #[getset(get = "pub")]
  datafusion_context: DfSessionContext,
  execution_plan_extensions: Arc<Vec<ExecutionPlanExtension>>,
}

impl Transaction {
  #[inline]
  pub async fn create_verified_logical_plan(
    &self,
    mut stmt: Box<SQLStatement>,
  ) -> Result<LogicalPlan> {
    let state = self.datafusion_context.state();

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
    // Check if the current session can execute the given statement
    if !self.session_config.privilege.can_execute(stmt.as_ref()) {
      return Err(Error::InsufficientPrivilege);
    }

    let stmt_type = StatementType::from(stmt.as_ref());
    let custom_plan = DEFAULT_EXTENSIONS
      .iter()
      .chain(self.execution_plan_extensions.iter())
      .find_map(|ext| ext(&self.session_state, &self, &stmt).transpose())
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
    let df = self
      .datafusion_context
      .execute_logical_plan(plan.clone())
      .await?;
    let physical_plan = df.create_physical_plan().await?;
    self.execute_stream(stmt_type, physical_plan).await
  }

  #[inline]
  pub fn closed(&self) -> bool {
    self.storage_transaction.closed()
  }

  #[inline]
  pub fn commit(self) -> Result<()> {
    self.storage_transaction.commit()
  }

  #[inline]
  pub fn rollback(self) -> Result<()> {
    self.storage_transaction.rollback()
  }

  #[inline]
  async fn execute_stream(
    &self,
    stmt_type: &StatementType,
    physical_plan: Arc<dyn ExecutionPlan>,
  ) -> Result<ExecutionResponse> {
    let response = execute_stream(
      physical_plan.clone(),
      self.datafusion_context.task_ctx().into(),
    )?;
    ExecutionResponse::from_stream(stmt_type, response).await
  }
}
