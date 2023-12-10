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

use super::{RecordBatch, RecordBatchStream};
use crate::execution::filter::Filter;
use crate::execution::iterators::heap_iterator::HeapIterator;
use crate::execution::iterators::unique_index_iterator::UniqueIndexIterator;
use crate::schema::{DataFrame, DataType, Table};
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
    // #[derivative(Debug = "ignore")]
    transaction: Transaction,
  ) -> Result<RecordBatch, DataFusionError> {
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

    let columns = schema
      .fields
      .iter()
      .map(|field| {
        (
          field.name().clone(),
          DataType::try_from(field.data_type()).unwrap(),
        )
      })
      .collect();
    let mut dataframe =
    // TODO: customize the DF capacity based on statistics
      DataFrame::with_capacity(limit.unwrap_or(1_000), columns);
    if let Some(index) = maybe_use_index {
      if index.is_unique() {
        UniqueIndexIterator::new(
          &storage,
          &table,
          index,
          &filters,
          &column_projection,
        )
        .fill_into(&mut dataframe)?;
      } else {
        unimplemented!()
      }
    } else {
      HeapIterator::new(&storage, &table, &column_projection)
        .fill_into(&mut dataframe)?;
    };

    // TODO: try if sending rows in batches improves per
    Ok(dataframe.to_record_batch(schema)?)
  }
}
