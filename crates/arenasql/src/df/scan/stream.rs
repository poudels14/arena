use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use datafusion::arrow::datatypes::SchemaRef;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::physical_plan::RecordBatchStream;
use futures::Stream;

use crate::schema::{RowConverter, SerializedCell, Table};
use crate::storage::{Serializer, Transaction};
use crate::table_rows_prefix_key;

pub struct RowStream {
  pub(super) table: Arc<Table>,
  pub(super) schema: SchemaRef,
  pub(super) transaction: Arc<dyn Transaction>,
  pub(super) serializer: Serializer,
  pub(super) done: bool,
}

impl RecordBatchStream for RowStream {
  fn schema(&self) -> SchemaRef {
    self.schema.clone()
  }
}

impl Stream for RowStream {
  type Item = datafusion::error::Result<RecordBatch>;

  fn poll_next(
    mut self: Pin<&mut Self>,
    _cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    if self.done {
      return Poll::Ready(None);
    }
    let raw_rows = self
      .transaction
      .scan(&table_rows_prefix_key!(self.table.id))
      .unwrap();

    if raw_rows.is_empty() {
      self.done = true;
      return Poll::Ready(None);
    }

    let rows = raw_rows
      .iter()
      .map(|(_key, value)| {
        self
          .serializer
          .deserialize::<Vec<SerializedCell<Vec<u8>>>>(&value)
          .unwrap()
      })
      .collect::<Vec<Vec<SerializedCell<Vec<u8>>>>>();

    let col_arrays =
      RowConverter::convert_to_columns(&self.table, &self.schema, &rows);

    self.done = true;
    return Poll::Ready(Some(Ok(RecordBatch::try_new(
      self.schema(),
      col_arrays,
    )?)));
  }
}
