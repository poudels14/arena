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
use derive_builder::Builder;
use futures::StreamExt;

use crate::datafusion::{RecordBatch, RecordBatchStream};
use crate::execution::filter::Filter;
use crate::execution::iterators::{HeapIterator, IndexIterator};
use crate::execution::TransactionHandle;
use crate::schema::{DataFrame, DataType, Table};

#[derive(Derivative, Clone, Builder)]
#[derivative(Debug)]
pub struct TableScaner {
  table: Arc<Table>,
  /// vec of selected columns by index
  projection: Vec<usize>,
  projected_schema: SchemaRef,
  #[derivative(Debug = "ignore")]
  pub transaction: TransactionHandle,
  filters: Vec<Filter>,
  limit: Option<usize>,
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
    let scan_fut = Self::scan_table(
      self.table.clone(),
      self.projection.clone(),
      self.schema(),
      self.filters.clone(),
      self.limit.clone(),
      self.transaction.clone(),
    );
    let stream = futures::stream::once(async move { scan_fut.await }).boxed();
    Ok(Box::pin(RecordBatchStreamAdapter::new(
      self.schema(),
      stream,
    )))
  }

  fn statistics(&self) -> Result<Statistics, DataFusionError> {
    Ok(Statistics::new_unknown(&self.projected_schema))
  }
}

impl TableScaner {
  pub async fn scan_table(
    table: Arc<Table>,
    // List of selected column indexes
    column_projection: Vec<usize>,
    schema: SchemaRef,
    filters: Vec<Filter>,
    limit: Option<usize>,
    transaction: TransactionHandle,
  ) -> Result<RecordBatch, DataFusionError> {
    // Only pass in physical columns since virutal columns like
    // ctid are default included
    let physical_columns: Vec<(String, DataType)> = column_projection
      .iter()
      .zip(schema.fields.iter())
      .map(|(_, field)| {
        (field.name().clone(), DataType::from_field(field).unwrap())
      })
      .collect();

    // Don't include virtual columns in column projection
    // since they are auto added
    let column_projection = column_projection
      .into_iter()
      .filter(|idx| *idx < table.columns.len())
      .collect::<Vec<usize>>();

    let storage = transaction.lock(false)?;
    let index_with_lowest_cost =
      Filter::find_index_with_lowest_cost(&table.indexes, &filters);

    let maybe_use_index = index_with_lowest_cost.or_else(|| {
      // If an index with lowest cost isn't found, check if there's
      // an index that has all the columns the query needs
      // TODO: what if there are more than one index with all columns?
      table.indexes.iter().find(|index| {
        let index_cols = index.columns();
        column_projection
          .iter()
          .all(|proj| index_cols.contains(proj))
      })
    });

    // TODO: customize the DF capacity based on statistics
    let mut dataframe =
      DataFrame::with_capacity(limit.unwrap_or(1_000), physical_columns);

    // TODO: if some filter is used, use index that has all the columns
    // from the filter even if all the selected columns are not in the index
    if let Some(index) = maybe_use_index {
      IndexIterator::new(&storage, &table, index, &filters, &column_projection)
        .fill_into(&mut dataframe)?;
    } else {
      HeapIterator::new(&storage, &table, &column_projection)
        .fill_into(&mut dataframe)?;
    };

    // TODO: try if sending rows in batches improves per
    Ok(dataframe.to_record_batch(schema)?)
  }
}
