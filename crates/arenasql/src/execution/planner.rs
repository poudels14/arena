use std::sync::Arc;

use async_trait::async_trait;
use datafusion::error::{DataFusionError, Result};
use datafusion::execution::context::{QueryPlanner, SessionState};
use datafusion::logical_expr::{LogicalPlan, WriteOp};
use datafusion::physical_plan::ExecutionPlan;
use datafusion::physical_planner::DefaultPhysicalPlanner;
use datafusion::physical_planner::PhysicalPlanner;

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
    session_state: &SessionState,
  ) -> Result<Arc<dyn ExecutionPlan>> {
    match logical_plan {
      LogicalPlan::EmptyRelation(_) => {}
      LogicalPlan::Dml(stmt) => {
        match stmt.op {
          WriteOp::InsertInto => {}
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
      .create_physical_plan(logical_plan, session_state)
      .await
  }
}
