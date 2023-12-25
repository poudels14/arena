use std::sync::Arc;

use datafusion::execution::context::SessionConfig as DfSessionConfig;
use derivative::Derivative;
use sqlparser::ast::Statement as SQLStatement;

use super::state::SessionState;
use super::transaction::Transaction;
use super::{response::ExecutionResponse, SessionConfig};
use crate::Result;

pub static DEFAULT_SCHEMA_NAME: &'static str = "public";

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct SessionContext {
  #[derivative(Debug = "ignore")]
  pub config: Arc<SessionConfig>,
  pub state: Arc<SessionState>,
  df_session_config: DfSessionConfig,
}

impl SessionContext {
  pub fn new(config: SessionConfig, state: SessionState) -> Self {
    let mut df_session_config = DfSessionConfig::new()
      .with_information_schema(false)
      .with_default_catalog_and_schema(
        config.catalog.as_ref(),
        DEFAULT_SCHEMA_NAME,
      )
      .with_create_default_catalog_and_schema(false);
    df_session_config.options_mut().sql_parser.dialect =
      "PostgreSQL".to_owned();

    Self {
      config: Arc::new(config),
      state: Arc::new(state),
      df_session_config,
    }
  }

  pub async fn execute(
    &self,
    stmts: Vec<Box<SQLStatement>>,
  ) -> Result<Vec<ExecutionResponse>> {
    let mut stmt_results = Vec::with_capacity(stmts.len());
    let transaction = self.begin_transaction()?;
    for stmt in stmts {
      stmt_results.push(transaction.execute(stmt).await?)
    }
    Ok(stmt_results)
  }

  pub fn begin_transaction(&self) -> Result<Transaction> {
    Transaction::new(
      self.config.clone(),
      self.state.clone(),
      self.df_session_config.clone(),
    )
  }
}
