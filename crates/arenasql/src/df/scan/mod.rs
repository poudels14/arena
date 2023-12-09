pub(crate) mod filter;
pub(crate) mod heap_iterator;
mod stream;
pub(crate) mod unique_index_iterator;

use std::any::Any;
use std::sync::Arc;

use datafusion::arrow::datatypes::SchemaRef;
use datafusion::error::DataFusionError;
use datafusion::execution::TaskContext;
use datafusion::physical_expr::PhysicalSortExpr;
use datafusion::physical_plan::stream::RecordBatchStreamAdapter;
use datafusion::physical_plan::{
  DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning, Statistics,
};
use derivative::Derivative;
use futures::StreamExt;

use self::filter::Filter;
pub use self::stream::RowsStream;
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
  pub(crate) filters: Vec<Filter>,
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
    _context: Arc<TaskContext>,
  ) -> Result<RecordBatchStream, DataFusionError> {
    let mut row_stream = RowsStream {
      table: self.table.clone(),
      schema: self.schema(),
      projection: self.projection.clone(),
      filters: self.filters.clone(),
      transaction: self.transaction.clone(),
      done: false,
    };

    let schema = self.schema();
    let stream =
      futures::stream::once(async move { row_stream.scan_table().await })
        .boxed();
    Ok(Box::pin(RecordBatchStreamAdapter::new(schema, stream)))
  }

  fn statistics(&self) -> Result<Statistics, DataFusionError> {
    Ok(Statistics::new_unknown(&self.projected_schema))
  }
}
