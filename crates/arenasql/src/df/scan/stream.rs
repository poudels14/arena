use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use datafusion::arrow::datatypes::SchemaRef;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::error::Result;
use datafusion::physical_plan::RecordBatchStream;
use futures::Stream;

use crate::schema::{ColumnArrayBuilder, SerializedCell, Table};
use crate::storage::{Serializer, Transaction};
use crate::{df_execution_error, table_rows_prefix_key};

pub struct RowStream {
  pub(super) table: Arc<Table>,
  pub(crate) projection: Vec<usize>,
  pub(super) schema: SchemaRef,
  pub(super) transaction: Transaction,
  pub(super) serializer: Serializer,
  pub(super) done: bool,
}

impl RowStream {
  fn poll_data(&mut self) -> Result<Option<RecordBatch>> {
    let transaction = self.transaction.lock();

    let mut raw_rows = transaction
      .scan_raw(&table_rows_prefix_key!(self.table.id))
      .map_err(|e| df_execution_error!("Storage error: {}", e.to_string()))?;

    let mut column_list_builders: Vec<ColumnArrayBuilder> = self
      .projection
      .iter()
      .map(|idx| {
        // TODO(sagar): pass in limit and use it as capacity when possible
        ColumnArrayBuilder::from(&self.table.columns[*idx].data_type, 5_000)
      })
      .collect();

    while let Some((_key, value)) = raw_rows.get() {
      let row = self
        .serializer
        .deserialize::<Vec<SerializedCell<&[u8]>>>(value)
        .map_err(|e| {
          df_execution_error!("Serialization error: {}", e.to_string())
        })?;
      self.projection.iter().for_each(|idx| {
        column_list_builders[*idx].append(unsafe { row.get_unchecked(*idx) });
      });
      raw_rows.next();
    }

    let col_arrays = column_list_builders
      .into_iter()
      .map(|b| b.finish())
      .collect();

    drop(transaction);
    self.done = true;
    Ok(Some(RecordBatch::try_new(self.schema(), col_arrays)?))
  }
}

impl RecordBatchStream for RowStream {
  fn schema(&self) -> SchemaRef {
    self.schema.clone()
  }
}

impl Stream for RowStream {
  type Item = Result<RecordBatch>;

  fn poll_next(
    mut self: Pin<&mut Self>,
    _cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    if self.done {
      return Poll::Ready(None);
    }
    return Poll::Ready(self.poll_data().transpose());
  }
}
