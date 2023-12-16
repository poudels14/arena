use std::sync::Arc;

use async_trait::async_trait;
use datafusion::error::{DataFusionError, Result};
use datafusion::execution::context::{QueryPlanner, SessionState};
use datafusion::logical_expr::{DmlStatement, LogicalPlan, WriteOp};
use datafusion::physical_plan::ExecutionPlan;
use datafusion::physical_planner::DefaultPhysicalPlanner;
use datafusion::physical_planner::PhysicalPlanner;

use crate::df::providers;
use crate::error::Error;

pub struct ArenaQueryPlanner {
  df_planner: DefaultPhysicalPlanner,
}

impl ArenaQueryPlanner {
  pub fn new() -> Self {
    Self {
      df_planner: DefaultPhysicalPlanner::default(),
    }
  }
}

#[async_trait]
impl QueryPlanner for ArenaQueryPlanner {
  async fn create_physical_plan(
    &self,
    logical_plan: &LogicalPlan,
    state: &SessionState,
  ) -> Result<Arc<dyn ExecutionPlan>> {
    match logical_plan {
      LogicalPlan::EmptyRelation(_) => {}
      LogicalPlan::Dml(DmlStatement {
        table_name,
        op,
        input,
        ..
      }) => {
        match op {
          WriteOp::InsertInto => {}
          WriteOp::Delete => {
            let scanner_plan =
              self.df_planner.create_physical_plan(&input, state).await?;
            let config_options = state.config_options();
            let catalog_name = table_name
              .catalog()
              .unwrap_or_else(|| &config_options.catalog.default_catalog);

            let schema_name = table_name
              .schema()
              .unwrap_or_else(|| &config_options.catalog.default_schema);

            let schema_provider = state
              .catalog_list()
              .catalog(catalog_name)
              // Catalog must exist!
              .unwrap()
              .schema(schema_name)
              .ok_or_else(|| {
                Error::SchemaDoesntExist(schema_name.to_owned())
              })?;

            let table_provider =
              schema_provider.table(table_name.table()).await.ok_or_else(
                || Error::RelationDoesntExist(table_name.table().to_owned()),
              )?;

            let table_provider = table_provider
              .as_any()
              .downcast_ref::<providers::table::TableProvider>()
              .unwrap();

            return table_provider.delete(scanner_plan).await;
          }
          _ => {
            return Err(DataFusionError::NotImplemented(
              "Unsupported Dml query".to_owned(),
            ))
          }
        };
      }
      LogicalPlan::Ddl(_stmt) => {
        panic!();
      }
      _ => {}
    }
    self
      .df_planner
      .create_physical_plan(logical_plan, state)
      .await
  }
}
