mod insert;
pub mod providers;
mod scan;

use std::sync::Arc;

pub use datafusion::catalog::CatalogList;
use datafusion::execution::context::SQLOptions;
use datafusion::execution::context::SessionConfig;
use datafusion::execution::context::SessionContext;
use datafusion::execution::context::SessionState;
use datafusion::execution::runtime_env::RuntimeEnv as DfRuntimeEnv;
use datafusion::physical_plan::collect;

use super::runtime::RuntimeEnv;
use super::storage::Transaction;
use crate::ast;
use crate::Result;

pub async fn execute(
  _runtime: &RuntimeEnv,
  _txn: Arc<dyn Transaction>,
  catlog_list: Arc<dyn CatalogList>,
  sql: &str,
) -> Result<()> {
  let ast = ast::parse(sql)?;

  let runtime = Arc::new(DfRuntimeEnv::default());
  let mut config_options = SessionConfig::new()
    .with_information_schema(false)
    .with_default_catalog_and_schema("arena", "public")
    .with_create_default_catalog_and_schema(false);
  config_options.options_mut().sql_parser.dialect = "PostgreSQL".to_owned();
  let state = SessionState::new_with_config_rt_and_catalog_list(
    config_options,
    runtime,
    catlog_list,
  );

  let sql_ctx = SessionContext::new_with_state(state);

  let state = sql_ctx.state();
  for stmt in ast {
    let sql_options = SQLOptions::new();
    let plan = state
      .statement_to_plan(datafusion::sql::parser::Statement::Statement(
        stmt.into(),
      ))
      .await?;

    sql_options.verify_plan(&plan)?;
    let df = sql_ctx.execute_logical_plan(plan).await?;
    let plan = df.create_physical_plan().await.unwrap();

    let _result = collect(plan.clone(), sql_ctx.task_ctx().into())
      .await
      .unwrap();
  }

  Ok(())
}
