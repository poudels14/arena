use std::sync::Arc;

use datafusion::arrow::record_batch::RecordBatch;
use datafusion::execution::context::{
  SQLOptions, SessionConfig as DfSessionConfig,
  SessionContext as DfSessionContext, SessionState as DfSessionState,
};
use sqlparser::ast::Statement as SQLStatement;

use super::config::TaskConfig;
use super::transaction::Transaction;
use super::SessionConfig;
use crate::{storage, Error, Result};

#[derive(Clone)]
pub struct SessionContext {
  config: Arc<SessionConfig>,
  df_session_config: DfSessionConfig,
}

impl SessionContext {
  pub fn with_config(config: SessionConfig) -> Self {
    let mut df_session_config = DfSessionConfig::new()
      .with_information_schema(false)
      .with_default_catalog_and_schema(&config.catalog, &config.schema)
      .with_create_default_catalog_and_schema(false)
      .with_extension(Arc::new(TaskConfig {
        runtime: config.runtime.clone(),
        serializer: config.serializer.clone(),
      }));
    df_session_config.options_mut().sql_parser.dialect =
      "PostgreSQL".to_owned();

    Self {
      config: Arc::new(config),
      df_session_config,
    }
  }

  pub async fn execute(
    &self,
    stmts: Vec<SQLStatement>,
  ) -> Result<Vec<Vec<RecordBatch>>> {
    let mut stmt_results = Vec::with_capacity(stmts.len());
    for stmt in stmts {
      let txn = self.begin_transaction()?;
      stmt_results.push(txn.execute(stmt).await?)
    }
    Ok(stmt_results)
  }

  pub fn begin_transaction(&self) -> Result<Transaction> {
    let txn = storage::Transaction::new(
      self.config.storage_provider.begin_transaction()?,
    );
    let catalog_list = self
      .config
      .catalog_list_provider
      .get_catalog_list(txn.clone());
    if catalog_list.is_none() {
      return Err(Error::SystemError("Valid catalog provider must be set"));
    }
    let state = DfSessionState::new_with_config_rt_and_catalog_list(
      self.df_session_config.clone(),
      self.config.df_runtime.clone(),
      catalog_list.unwrap(),
    );

    let sql_options = SQLOptions::new();
    Ok(Transaction {
      txn,
      sql_options,
      ctxt: DfSessionContext::new_with_state(state),
    })
  }
}
