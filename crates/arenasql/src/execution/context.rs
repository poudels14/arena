use std::sync::Arc;

use datafusion::execution::context::{
  SQLOptions, SessionConfig as DfSessionConfig,
  SessionContext as DfSessionContext, SessionState as DfSessionState,
};
use derivative::Derivative;
use sqlparser::ast::Statement as SQLStatement;

use super::custom_functions;
use super::planner::ArenaQueryPlanner;
use super::state::SessionState;
use super::transaction::Transaction;
use super::{response::ExecutionResponse, SessionConfig};
use crate::{Error, Result};

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
    for stmt in stmts {
      let txn = self.begin_transaction()?;
      stmt_results.push(txn.execute(stmt).await?)
    }
    Ok(stmt_results)
  }

  pub fn begin_transaction(&self) -> Result<Transaction> {
    let storage_transaction = self
      .config
      .storage_factory
      .being_transaction(self.config.schemas.clone())?;

    let catalog_list = self.config.catalog_list_provider.get_catalog_list(
      self.config.catalog.clone(),
      self.config.schemas.clone(),
      storage_transaction.clone(),
    );
    if catalog_list.is_none() {
      return Err(Error::InternalError(
        "Valid catalog provider must be set".to_owned(),
      ));
    }
    let state = DfSessionState::new_with_config_rt_and_catalog_list(
      self.df_session_config.clone(),
      self.config.df_runtime.clone(),
      catalog_list.unwrap(),
    )
    .with_query_planner(Arc::new(ArenaQueryPlanner::new()));

    let session_context = DfSessionContext::new_with_state(state);
    custom_functions::register_all(&session_context);

    let sql_options = SQLOptions::new();
    Ok(Transaction::new(
      self.config.clone(),
      self.state.clone(),
      storage_transaction,
      sql_options,
      session_context,
      self.config.execution_plan_extensions.clone(),
    ))
  }
}
