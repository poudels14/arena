use std::sync::Arc;

use async_trait::async_trait;
use datafusion::error::{DataFusionError, Result};
use datafusion::execution::context::{QueryPlanner, SessionState};
use datafusion::logical_expr::{DmlStatement, LogicalPlan, WriteOp};
use datafusion::physical_plan::ExecutionPlan;
use datafusion::physical_planner::DefaultPhysicalPlanner;
use datafusion::physical_planner::PhysicalPlanner;

use crate::df::providers::{self, get_schema_provider, get_table_ref};
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
          WriteOp::InsertInto | WriteOp::InsertOverwrite => {}
          WriteOp::Delete | WriteOp::Update => {
            let table_name = table_name.to_string();
            let table_ref = get_table_ref(&state, &table_name);
            let schema_provider = get_schema_provider(state, &table_ref)?;

            let table_provider = schema_provider
              .table(&table_name)
              .await
              .ok_or_else(|| Error::RelationDoesntExist(table_name))?;

            let table_provider = table_provider
              .as_any()
              .downcast_ref::<providers::table::TableProvider>()
              .unwrap();

            let scanner_plan =
              self.df_planner.create_physical_plan(&input, &state).await?;
            if *op == WriteOp::Delete {
              return table_provider.delete(scanner_plan).await;
            } else if *op == WriteOp::Update {
              return table_provider.update(scanner_plan).await;
            }
          }
          _ => {
            return Err(DataFusionError::NotImplemented(
              "Unsupported query".to_owned(),
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
