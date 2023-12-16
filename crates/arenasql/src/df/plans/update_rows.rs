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
pub struct UpdateRowsExecutionPlan {
  table: Arc<Table>,
  pub(crate) scanner: Arc<dyn ExecutionPlan>,
  #[builder(setter(skip), default = "self.default_schema()")]
  schema: SchemaRef,
  #[derivative(Debug = "ignore")]
  transaction: Transaction,
}

impl UpdateRowsExecutionPlanBuilder {
  fn default_schema(&self) -> SchemaRef {
    Arc::new(Schema::new(vec![Field::new(
      "count",
      DataType::UInt64,
      false,
    )]))
  }
}

impl DisplayAs for UpdateRowsExecutionPlan {
  fn fmt_as(
    &self,
    _t: DisplayFormatType,
    f: &mut fmt::Formatter,
  ) -> fmt::Result {
    // TODO
    write!(f, "{:?}", self)
  }
}

impl ExecutionPlan for UpdateRowsExecutionPlan {
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
    let update_fut = async move {
      let stream = table_scanner.execute(partition, context)?;

      let modified_rows_count: usize = stream
        .map(move |batch| {
          let transaction = transaction.lock(true)?;
          batch.and_then(|batch| {
            let rows =
              rowconverter::convert_to_rows(&table, &batch, true).unwrap();
            rows
              .iter()
              .map(|new_row| {
                let row_id = &new_row[new_row.len() - 1];
                let row_id_bytes =
                  RowId::serialize_u64(row_id.as_u64().unwrap());
                // Datafusion doesn't include non-updated rows, so query it again
                // TODO: figure out how to prevent querying the row again here
                let old_row =
                  transaction.get_row(&table, &row_id_bytes)?.unwrap();
                for table_index in &table.indexes {
                  transaction.delete_row_from_index(&table_index, &old_row)?;
                  transaction.add_row_to_index(
                    &table,
                    &table_index,
                    &row_id_bytes,
                    &new_row,
                  )?;
                }
                transaction.delete_row(&table, &row_id_bytes)?;
                transaction.insert_row(&table, &row_id_bytes, new_row)?;

                Ok(())
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

    let stream = futures::stream::once(async move { update_fut.await }).boxed();
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
