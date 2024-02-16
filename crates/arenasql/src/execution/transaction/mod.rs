mod handle;
mod lock;

use std::borrow::BorrowMut;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use datafusion::common::DFSchema;
use datafusion::execution::context::{
  SQLOptions, SessionConfig as DfSessionConfig,
  SessionContext as DfSessionContext, SessionState as DfSessionState,
};
use datafusion::logical_expr::{
  Extension, LogicalPlan, Statement as LogicalStatement, TransactionAccessMode,
  TransactionIsolationLevel, TransactionStart,
};
use datafusion::physical_plan::{execute_stream, ExecutionPlan};
use getset::Getters;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use sqlparser::ast::Statement as SQLStatement;

use super::execution_plan::CustomPlanAdapter;
use super::planner::ArenaQueryPlanner;
use super::response::ExecutionResponse;
use super::{custom_functions, ExecutionPlanExtension};
use super::{SessionConfig, SessionState};
use crate::ast::statement::StatementType;
use crate::df::plans::{
  self, alter_table, create_index, insert_rows, set_parameter,
};
use crate::{ast, Error, Result};

pub use handle::TransactionHandle;
pub use lock::TransactionLock;

static TRANSACTION_ID: Lazy<Arc<AtomicUsize>> =
  Lazy::new(|| Arc::new(AtomicUsize::new(1)));

pub const DEFAULT_EXTENSIONS: Lazy<Arc<Vec<ExecutionPlanExtension>>> =
  Lazy::new(|| {
    Arc::new(vec![
      Arc::new(create_index::extension),
      Arc::new(plans::advisory_lock::extension),
      Arc::new(set_parameter::extension),
      Arc::new(alter_table::extension),
    ])
  });

#[derive(Getters, Clone)]
pub struct Transaction {
  #[allow(unused)]
  pub id: usize,
  #[getset(get = "pub")]
  session_config: Arc<SessionConfig>,
  #[getset(get = "pub")]
  session_state: Arc<RwLock<SessionState>>,
  sql_options: SQLOptions,
  df_session_config: Arc<DfSessionConfig>,
  #[getset(get = "pub")]
  datafusion_context: Arc<DfSessionContext>,
  pub(crate) handle: TransactionHandle,
}

impl Transaction {
  #[tracing::instrument(skip_all, level = "TRACE")]
  pub(crate) fn new(
    session_config: Arc<SessionConfig>,
    session_state: Arc<RwLock<SessionState>>,
    df_session_config: DfSessionConfig,
  ) -> Result<Self> {
    let handle = session_config
      .storage_factory
      .create_new_transaction_handle(session_config.schemas.clone())?;
    Ok(Self::new_with_handle(
      TRANSACTION_ID.fetch_add(1, Ordering::AcqRel),
      handle,
      session_config,
      session_state,
      df_session_config,
    ))
  }

  #[tracing::instrument(
    skip(handle, session_config, session_state, df_session_config),
    level = "TRACE"
  )]
  pub(crate) fn new_with_handle(
    id: usize,
    handle: TransactionHandle,
    session_config: Arc<SessionConfig>,
    session_state: Arc<RwLock<SessionState>>,
    df_session_config: DfSessionConfig,
  ) -> Self {
    let catalog_list = session_config.catalog_list_provider.get_catalog_list(
      session_config.catalog.clone(),
      session_config.schemas.clone(),
      handle.clone(),
    );

    let state = DfSessionState::new_with_config_rt_and_catalog_list(
      df_session_config.clone(),
      session_config.df_runtime.clone(),
      catalog_list,
    )
    .with_query_planner(Arc::new(ArenaQueryPlanner::new()));

    let datafusion_context = DfSessionContext::new_with_state(state);
    custom_functions::register_all(&datafusion_context);

    let sql_options = SQLOptions::new();

    Self {
      id,
      session_config,
      session_state,
      sql_options,
      df_session_config: df_session_config.into(),
      datafusion_context: datafusion_context.into(),
      handle,
    }
  }

  #[inline]
  pub fn handle(&self) -> &TransactionHandle {
    &self.handle
  }

  #[tracing::instrument(skip_all, level = "TRACE")]
  #[inline]
  pub async fn execute_sql(&self, sql: &str) -> Result<ExecutionResponse> {
    let mut stmts = crate::ast::parse(sql)?;
    if stmts.len() != 1 {
      return Err(Error::UnsupportedOperation(
        "In a transaction, one and only one statement should be executed"
          .to_owned(),
      ));
    }
    self.execute(stmts.pop().unwrap().into()).await
  }

  #[tracing::instrument(skip_all, err, level = "TRACE")]
  #[inline]
  pub async fn create_verified_logical_plan(
    &self,
    mut stmt: Box<SQLStatement>,
  ) -> Result<LogicalPlan> {
    // Check if the current session can execute the given statement
    if !self.session_config.privilege.can_execute(stmt.as_ref()) {
      return Err(Error::InsufficientPrivilege);
    }
    let state = self.datafusion_context.state();
    let stmt_type = StatementType::from(stmt.as_ref());
    tracing::trace!(
      "transaction_id = {:?}, stmt_type = {:?}",
      self.id,
      stmt_type.to_string()
    );
    // Modify stmt if needed
    // THIS IS A HACK needed because table scan needs to return rowid
    // for delete/update
    match stmt_type.is_insert() {
      true => {
        let stmt: &mut SQLStatement = stmt.borrow_mut();
        insert_rows::set_explicit_columns_in_insert_query(&state, stmt).await?;
      }
      _ => {}
    };

    let mut statement = stmt.borrow_mut();
    if stmt_type == StatementType::Create {
      // TODO: remove this when datafusion support custom data types
      // replace data type to anything that datafusion doesn't throw error for
      ast::cast_unsupported_data_types(&mut statement)?;
    }

    let custom_plan = DEFAULT_EXTENSIONS
      .iter()
      .chain(self.session_config.execution_plan_extensions.iter())
      .find_map(|ext| ext(&self, &stmt).transpose())
      .transpose()?;
    if let Some(plan) = custom_plan {
      tracing::trace!("using custom plan",);
      return Ok(LogicalPlan::Extension(Extension {
        node: Arc::new(CustomPlanAdapter::create(plan)),
      }));
    }

    if stmt_type.is_begin() {
      return Ok(LogicalPlan::Statement(LogicalStatement::TransactionStart(
        TransactionStart {
          access_mode: TransactionAccessMode::ReadWrite,
          schema: DFSchema::empty().into(),
          isolation_level: TransactionIsolationLevel::Serializable,
        },
      )));
    }

    // TODO: creating physical plan from SQL is expensive
    // look into caching physical plans
    tracing::trace!("creating logical plan from statement",);
    let plan = state
      .statement_to_plan(datafusion::sql::parser::Statement::Statement(stmt))
      .await?;
    self.sql_options.verify_plan(&plan)?;
    Ok(plan)
  }

  #[tracing::instrument(skip_all, level = "TRACE")]
  pub async fn execute(
    &self,
    stmt: Box<SQLStatement>,
  ) -> Result<ExecutionResponse> {
    tracing::trace!("transaction_id = {:?}", self.id);
    let logical_plan = self.create_verified_logical_plan(stmt.clone()).await?;
    let stmt_type = StatementType::from(stmt.as_ref());
    self
      .execute_logical_plan(&stmt_type, stmt, logical_plan)
      .await
  }

  #[tracing::instrument(skip(self, stmt, plan), level = "TRACE")]
  #[inline]
  pub async fn execute_logical_plan(
    &self,
    stmt_type: &StatementType,
    stmt: Box<SQLStatement>,
    plan: LogicalPlan,
  ) -> Result<ExecutionResponse> {
    tracing::trace!("transaction_id = {:?}", self.id);
    if let LogicalPlan::Extension(extension) = plan {
      tracing::debug!("Using custom execution plan");
      return self
        .execute_stream(
          &stmt_type,
          Arc::new(
            extension
              .node
              .as_any()
              .downcast_ref::<CustomPlanAdapter>()
              .unwrap()
              .get_execution_plan(),
          ),
        )
        .await;
    };

    let mut statement = stmt;
    let mut txn = self;
    #[allow(unused)]
    let mut handle_ref = None;
    if *stmt_type == StatementType::Create {
      // NOTE: this is a hack to pass current query statement to the execution
      // plan so that execution plans can have access to sql data types instead
      // of just datafusion data types; datafusion doesn't support all datatypes
      // and we need to access the query to support custom data types like VECTOR,
      // JSONB, etc
      // TODO: remove this when datafusion support custom data types
      let mut txn_handle = txn.handle.clone();
      txn_handle.set_active_statement(Some(statement.clone().into()));

      // replace data type to anything that datafusion doesn't throw error for
      ast::cast_unsupported_data_types(&mut statement)?;
      handle_ref = Some(Self::new_with_handle(
        self.id,
        txn_handle,
        txn.session_config.clone(),
        txn.session_state.clone(),
        txn.df_session_config.as_ref().clone(),
      ));
      txn = handle_ref.as_mut().unwrap();
    }

    let df = txn
      .datafusion_context
      .execute_logical_plan(plan.clone())
      .await?;

    let physical_plan = df.create_physical_plan().await?;
    let result = txn.execute_stream(&stmt_type, physical_plan).await?;
    Ok(result)
  }

  #[tracing::instrument(skip_all, level = "TRACE")]
  #[inline]
  async fn execute_stream(
    &self,
    stmt_type: &StatementType,
    physical_plan: Arc<dyn ExecutionPlan>,
  ) -> Result<ExecutionResponse> {
    tracing::trace!("transaction_id = {:?}", self.id);
    let response = execute_stream(
      physical_plan.clone(),
      self.datafusion_context.task_ctx().into(),
    )?;
    ExecutionResponse::from_stream(stmt_type, response).await
  }

  #[tracing::instrument(skip_all, level = "TRACE")]
  #[inline]
  pub fn commit(self) -> Result<()> {
    tracing::trace!("transaction_id = {:?}", self.id);
    self.handle.commit()
  }

  #[tracing::instrument(skip_all, level = "TRACE")]
  #[inline]
  pub fn rollback(self) -> Result<()> {
    tracing::trace!("transaction_id = {:?}", self.id);
    self.handle.rollback()
  }
}
