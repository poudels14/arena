use std::sync::atomic::Ordering;
use std::sync::Arc;

use datafusion::execution::context::SessionConfig as DfSessionConfig;
use datafusion::logical_expr::LogicalPlan;
use datafusion::scalar::ScalarValue;
use derivative::Derivative;
use getset::Getters;
use parking_lot::{Mutex, RwLock};
use sqlparser::ast::Statement as SQLStatement;

use super::state::SessionState;
use super::transaction::Transaction;
use super::{response::ExecutionResponse, SessionConfig};
use crate::ast::statement::StatementType;
use crate::response::StreamCompletionHook;
use crate::{Error, Result};

pub static DEFAULT_SCHEMA_NAME: &'static str = "public";

#[derive(Clone, Derivative, Getters)]
#[derivative(Debug)]
pub struct SessionContext {
  #[derivative(Debug = "ignore")]
  pub config: Arc<SessionConfig>,
  pub state: Arc<RwLock<SessionState>>,
  df_session_config: Arc<DfSessionConfig>,

  #[derivative(Debug = "ignore")]
  active_transaction: Arc<Mutex<Option<Transaction>>>,
}

impl SessionContext {
  pub fn new(config: SessionConfig, state: SessionState) -> Result<Self> {
    let mut df_session_config = DfSessionConfig::new()
      .with_information_schema(config.enable_information_schema)
      .with_default_catalog_and_schema(
        config.catalog.as_ref(),
        DEFAULT_SCHEMA_NAME,
      )
      .with_create_default_catalog_and_schema(false);
    df_session_config.options_mut().sql_parser.dialect =
      "PostgreSQL".to_owned();

    let config = Arc::new(config);
    let state = Arc::new(RwLock::new(state));
    Ok(Self {
      config,
      state,
      df_session_config: Arc::new(df_session_config),
      active_transaction: Arc::new(Mutex::new(None)),
    })
  }

  /// The caller is responsible for committing the transaction returned.
  /// If not manually committed, the transaction will be rolled back.
  /// Instead of using this the transaction directly, execute query using
  /// `context.execute_sql(...)`
  pub unsafe fn get_or_create_active_transaction(&self) -> Transaction {
    let txn = self.active_transaction.lock().clone();
    match txn {
      Some(txn) => txn,
      None => self
        .new_active_transaction()
        .expect("Error creating new transaction"),
    }
  }

  #[tracing::instrument(skip_all, err, level = "TRACE")]
  #[inline]
  pub async fn execute_sql(&self, sql: &str) -> Result<Vec<ExecutionResponse>> {
    let stmts = crate::ast::parse(sql)?;
    let mut results = Vec::with_capacity(stmts.len());
    for stmt in stmts.into_iter() {
      let result = self.execute_statement(stmt.into(), None, None).await?;
      results.push(result);
    }
    Ok(results)
  }

  #[tracing::instrument(skip_all, err, level = "TRACE")]
  pub async fn execute_statement(
    &self,
    stmt: Box<SQLStatement>,
    logical_plan: Option<LogicalPlan>,
    params: Option<Vec<ScalarValue>>,
  ) -> Result<ExecutionResponse> {
    let stmt_type = StatementType::from(stmt.as_ref());
    tracing::trace!("{:?}", stmt_type);
    if stmt_type.is_begin() {
      let transaction = unsafe { self.get_or_create_active_transaction() };
      transaction.handle.is_chained().swap(true, Ordering::AcqRel);
      return Ok(ExecutionResponse::empty());
    } else if stmt_type.is_commit() {
      self.commit_active_transaction()?;
      return Ok(ExecutionResponse::empty());
    } else if stmt_type.is_rollback() {
      self.rollback_active_transaction()?;
      return Ok(ExecutionResponse::empty());
    }

    let transaction = unsafe { self.get_or_create_active_transaction() };
    let logical_plan = match logical_plan {
      Some(logical_plan) => logical_plan,
      None => {
        transaction
          .create_verified_logical_plan(stmt.clone())
          .await?
      }
    };

    let final_logical_plan = match params {
      Some(param_values) => logical_plan
        .with_param_values(param_values)
        .map_err(|e| Error::DataFusionError(e.into()))
        .expect(&format!(
          "Error replace_params_with_values at: {}:{}",
          file!(),
          line!()
        )),
      None => logical_plan,
    };

    let mut response = transaction
      .execute_logical_plan(&stmt_type, stmt, final_logical_plan)
      .await?;

    match stmt_type {
      // No need to commit/rollback transaction for query stmt type
      StatementType::Query => Ok(response),
      // Commit the transaction for execute query if it's not a chained
      // transaction. i.e. if it wasn't explicitly started by `BEGIN` command
      _ => match transaction.handle.is_chained().load(Ordering::Acquire) {
        true => Ok(response),
        false => {
          let transaction = self
            .active_transaction
            .lock()
            .take()
            .expect("Invalid active transaction");
          response.set_stream_completion_hook(StreamCompletionHook::new(
            Box::new(move || transaction.commit()),
          ))?;
          Ok(response)
        }
      },
    }
  }

  /// The caller is responsible for committing the transaction
  /// If not manually committed, the transaction will be rolled back
  #[tracing::instrument(skip_all, err, level = "TRACE")]
  pub unsafe fn create_new_active_transaction(&self) -> Result<Transaction> {
    self.new_active_transaction()
  }

  /// Replaces the active transaction of the context with the new
  /// transaction and returns the new transaction
  #[tracing::instrument(skip_all, level = "TRACE")]
  pub(crate) fn new_active_transaction(&self) -> Result<Transaction> {
    let new_transaction = Transaction::new(
      self.config.clone(),
      self.state.clone(),
      self.df_session_config.as_ref().clone(),
    )?;

    let mut transaction = self.active_transaction.lock();
    *transaction = Some(new_transaction.clone());
    Ok(new_transaction)
  }

  /// Commits the current transaction and create a new current transaction
  /// for the session
  #[tracing::instrument(skip_all, level = "TRACE")]
  pub fn commit_active_transaction(&self) -> Result<()> {
    let txn = self.active_transaction.lock().take().ok_or(
      Error::InvalidTransactionState("No active transaction".to_owned()),
    )?;
    txn.commit()?;
    Ok(())
  }

  /// Rollbacks the current transaction, return it
  /// and create a new current transaction for the session
  #[tracing::instrument(skip_all, level = "TRACE")]
  pub fn rollback_active_transaction(&self) -> Result<()> {
    let txn = self.active_transaction.lock().take().ok_or(
      Error::InvalidTransactionState("No active transaction".to_owned()),
    )?;
    txn.rollback()?;
    Ok(())
  }
}
