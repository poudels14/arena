use std::any::Any;
use std::fmt;
use std::sync::Arc;

use datafusion::arrow::array::UInt64Array;
use datafusion::arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::common::Statistics;
use datafusion::error::{DataFusionError, Result};
use datafusion::execution::TaskContext;
use datafusion::physical_expr::PhysicalSortExpr;
use datafusion::physical_plan::metrics::MetricsSet;
use datafusion::physical_plan::stream::RecordBatchStreamAdapter;
use datafusion::physical_plan::{
  DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning,
  SendableRecordBatchStream,
};
use derivative::Derivative;
use derive_builder::Builder;
use futures::{StreamExt, TryStreamExt};

use crate::schema::{RowId, Table};
use crate::storage::Transaction;
use crate::utils::rowconverter;
use crate::Error;

#[derive(Builder, Derivative)]
#[derivative(Debug)]
pub struct DeleteRowsExecutionPlan {
  table: Arc<Table>,
  pub(crate) scanner: Arc<dyn ExecutionPlan>,
  #[builder(setter(skip), default = "self.default_schema()")]
  schema: SchemaRef,
  #[derivative(Debug = "ignore")]
  transaction: Transaction,
}

impl DeleteRowsExecutionPlanBuilder {
  fn default_schema(&self) -> SchemaRef {
    Arc::new(Schema::new(vec![Field::new(
      "count",
      DataType::UInt64,
      false,
    )]))
  }
}

impl DisplayAs for DeleteRowsExecutionPlan {
  fn fmt_as(
    &self,
    _t: DisplayFormatType,
    f: &mut fmt::Formatter,
  ) -> fmt::Result {
    // TODO
    write!(f, "{:?}", self)
  }
}

impl ExecutionPlan for DeleteRowsExecutionPlan {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn execute(
    &self,
    partition: usize,
    context: Arc<TaskContext>,
  ) -> Result<SendableRecordBatchStream> {
    let schema = self.schema();
    let table = self.table.clone();

    let transaction = self.transaction.clone();
    let table_scanner = self.scanner.clone();
    let delete_fut = async move {
      let stream = table_scanner.execute(partition, context)?;

      let modified_rows_count: usize = stream
        .map(move |maybe_batch| {
          let transaction = transaction.lock(true)?;
          maybe_batch.and_then(|batch| {
            let rows = rowconverter::convert_to_rows(&table, &batch, true)?;
            rows
              .iter()
              .map(|row| {
                for table_index in &table.indexes {
                  transaction.delete_row_from_index(&table_index, row)?;
                }
                Ok(transaction.delete_row(
                  &table,
                  &RowId::serialize_u64(row[row.len() - 1].as_u64().unwrap()),
                )?)
              })
              .collect::<Result<Vec<()>>>()?;

            Ok(batch.num_rows())
          })
        })
        .try_collect::<Vec<usize>>()
        .await?
        .iter()
        .sum();

      Ok(
        RecordBatch::try_new(
          schema,
          vec![Arc::new(UInt64Array::from(vec![
            modified_rows_count as u64,
          ]))],
        )
        .map_err(|e| {
          Error::DataFusionError(DataFusionError::ArrowError(e).into())
        })?,
      )
    };

    let stream = futures::stream::once(async move { delete_fut.await }).boxed();
    Ok(Box::pin(RecordBatchStreamAdapter::new(
      self.schema(),
      stream,
    )))
  }

  fn schema(&self) -> SchemaRef {
    self.schema.clone()
  }

  fn with_new_children(
    self: Arc<Self>,
    _children: Vec<Arc<dyn ExecutionPlan>>,
  ) -> Result<Arc<dyn ExecutionPlan>> {
    unimplemented!()
  }

  fn children(&self) -> Vec<Arc<dyn ExecutionPlan>> {
    vec![self.scanner.clone()]
  }

  fn output_ordering(&self) -> Option<&[PhysicalSortExpr]> {
    None
  }

  fn output_partitioning(&self) -> Partitioning {
    Partitioning::UnknownPartitioning(1)
  }

  fn metrics(&self) -> Option<MetricsSet> {
    None
  }

  fn statistics(&self) -> Result<Statistics> {
    Ok(Statistics::new_unknown(&Schema::empty()))
  }
}
