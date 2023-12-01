mod stream;

use std::any::Any;
use std::sync::Arc;

use datafusion::arrow::datatypes::SchemaRef;
use datafusion::error::DataFusionError;
use datafusion::execution::TaskContext;
use datafusion::logical_expr::Expr;
use datafusion::physical_expr::PhysicalSortExpr;
use datafusion::physical_plan::{
  DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning, Statistics,
};
use derivative::Derivative;

pub use self::stream::RowsStream;
use super::execution::TaskConfig;
use super::RecordBatchStream;
use crate::schema::Table;
use crate::storage::Transaction;

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct TableScaner {
  pub(crate) table: Arc<Table>,
  /// vec of selected columns by index
  pub(crate) projection: Vec<usize>,
  pub(crate) projected_schema: SchemaRef,
  #[derivative(Debug = "ignore")]
  pub(crate) transaction: Transaction,
  pub(crate) filters: Vec<Expr>,
  pub(crate) limit: Option<usize>,
}

impl DisplayAs for TableScaner {
  fn fmt_as(
    &self,
    _t: DisplayFormatType,
    f: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(f, "TableScaner")
  }
}

impl ExecutionPlan for TableScaner {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn schema(&self) -> SchemaRef {
    self.projected_schema.clone()
  }

  fn output_partitioning(&self) -> Partitioning {
    Partitioning::UnknownPartitioning(1)
  }

  fn output_ordering(&self) -> Option<&[PhysicalSortExpr]> {
    None
  }

  fn children(&self) -> Vec<Arc<dyn ExecutionPlan>> {
    vec![]
  }

  fn with_new_children(
    self: Arc<Self>,
    _: Vec<Arc<dyn ExecutionPlan>>,
  ) -> Result<Arc<dyn ExecutionPlan>, DataFusionError> {
    Ok(self)
  }

  fn execute(
    &self,
    _partition: usize,
    context: Arc<TaskContext>,
  ) -> Result<RecordBatchStream, DataFusionError> {
    let task_config = context
      .session_config()
      .get_extension::<TaskConfig>()
      .unwrap();

    Ok(Box::pin(RowsStream {
      schema: self.schema(),
      projection: self.projection.clone(),
      table: self.table.clone(),
      transaction: self.transaction.clone(),
      serializer: task_config.serializer.clone(),
      done: false,
    }))
  }

  fn statistics(&self) -> Statistics {
    Statistics {
      num_rows: None,
      total_byte_size: None,
      column_statistics: None,
      is_exact: false,
    }
  }
}
