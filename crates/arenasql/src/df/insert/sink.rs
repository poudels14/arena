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

use crate::df::execution::TaskConfig;
use crate::df::RecordBatchStream;
use crate::schema::{RowConverter, Table};
use crate::storage::Transaction;
use crate::{
  df_execution_error, last_row_id_of_table_key, table_rows_prefix_key,
};

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
    context: &Arc<TaskContext>,
  ) -> Result<u64> {
    let task_config = context
      .session_config()
      .get_extension::<TaskConfig>()
      .unwrap();
    let mut modified_rows_count = 0;

    if let Some(batch) = data.next().await {
      if let Err(e) = batch {
        return Err(e);
      }
      let batch = batch?;
      let row_count = batch.num_rows();
      modified_rows_count += row_count;

      let rows = RowConverter::convert_to_rows(&self.table, &batch)?;

      let transaction = self.transaction.lock();
      for row in rows.iter() {
        let row_bytes =
          task_config.serializer.serialize(&row).map_err(|e| {
            df_execution_error!("Serialization error: {}", e.to_string())
          })?;
        transaction
          .atomic_update(
            &last_row_id_of_table_key!(self.table.id),
            &|old: Option<Vec<u8>>| {
              let new_row_id = old
                .map(|b| u64::from_be_bytes(b.try_into().unwrap()))
                .unwrap_or(0)
                + 1;
              new_row_id.to_be_bytes().to_vec()
            },
          )
          .and_then(|row_id| {
            transaction.put(
              &vec![table_rows_prefix_key!(self.table.id), row_id].concat(),
              &row_bytes,
            )
          })
          .map_err(|e| {
            df_execution_error!("Storage error: {}", e.to_string())
          })?;
      }
    }
    Ok(modified_rows_count as u64)
  }
}
