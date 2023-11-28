use datafusion::arrow::record_batch::RecordBatch;
use datafusion::execution::context::{
  SQLOptions, SessionContext as DfSessionContext,
};
use datafusion::physical_plan::collect;
use sqlparser::ast::Statement as SQLStatement;

use crate::{storage, Error, Result};

#[allow(unused)]
pub struct Transaction {
  pub(super) txn: storage::Transaction,
  pub(super) sql_options: SQLOptions,
  pub(super) ctxt: DfSessionContext,
}

impl Transaction {
  pub async fn execute_sql(&self, sql: &str) -> Result<Vec<RecordBatch>> {
    let mut stmts = crate::ast::parse(sql)?;
    if stmts.len() != 1 {
      return Err(Error::InvalidQuery(
        "In a transaction, one and only one statement should be executed"
          .to_owned(),
      ));
    }
    self.execute(stmts.pop().unwrap()).await
  }

  pub async fn execute(&self, stmt: SQLStatement) -> Result<Vec<RecordBatch>> {
    let plan = self
      .ctxt
      .state()
      .statement_to_plan(datafusion::sql::parser::Statement::Statement(
        stmt.into(),
      ))
      .await?;

    self.sql_options.verify_plan(&plan)?;
    let df = self.ctxt.execute_logical_plan(plan).await?;
    let plan = df.create_physical_plan().await.unwrap();

    return Ok(collect(plan.clone(), self.ctxt.task_ctx().into()).await?);
  }

  pub fn commit(self) -> Result<()> {
    self.txn.commit()
  }

  pub fn rollback(self) -> Result<()> {
    self.txn.rollback()
  }
}
