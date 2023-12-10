use std::any::Any;
use std::fmt::Formatter;
use std::sync::Arc;

use async_trait::async_trait;
use datafusion::arrow::datatypes::SchemaRef;
use datafusion::error::Result;
use datafusion::execution::TaskContext;
use datafusion::physical_plan::insert::DataSink;
use datafusion::physical_plan::metrics::MetricsSet;
use datafusion::physical_plan::{DisplayAs, DisplayFormatType};
use derivative::Derivative;
use futures::StreamExt;

use crate::df::RecordBatchStream;
use crate::schema::Table;
use crate::storage::Transaction;
use crate::utils::rowconverter;

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct Sink {
  pub table: Arc<Table>,
  pub schema: SchemaRef,
  #[derivative(Debug = "ignore")]
  pub transaction: Transaction,
}

impl DisplayAs for Sink {
  fn fmt_as(
    &self,
    t: DisplayFormatType,
    f: &mut Formatter,
  ) -> std::fmt::Result {
    match t {
      DisplayFormatType::Default | DisplayFormatType::Verbose => {
        write!(f, "{:?}", self)
      }
    }
  }
}

#[async_trait]
impl DataSink for Sink {
  fn as_any(&self) -> &dyn Any {
    return self;
  }

  fn metrics(&self) -> Option<MetricsSet> {
    None
  }

  async fn write_all(
    &self,
    mut data: RecordBatchStream,
    _context: &Arc<TaskContext>,
  ) -> Result<u64> {
    // TODO: acquire read lock on the table schema so that
    // the table schema can't be updated by any other transaction
    // when this write is happening
    let mut modified_rows_count = 0;

    if let Some(batch) = data.next().await {
      let batch = batch?;
      let row_count = batch.num_rows();
      modified_rows_count += row_count;

      let rows = rowconverter::convert_to_rows(&self.table, &batch)?;
      let storage_handler = self.transaction.lock(true)?;
      for row in rows.iter() {
        let row_id = storage_handler.generate_next_row_id(&self.table)?;
        for table_index in &self.table.indexes {
          storage_handler.add_row_to_index(
            &self.table,
            &table_index,
            &row_id,
            row,
          )?;
        }
        storage_handler.insert_row(&self.table, &row_id, &row)?;
      }
    }
    Ok(modified_rows_count as u64)
  }
}
