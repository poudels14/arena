use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use datafusion::arrow::datatypes::SchemaRef;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::physical_plan::RecordBatchStream;
use futures::Stream;

use crate::schema::{DataWithValue, RowConverter, Table};
use crate::storage::{self, Transaction};

pub struct RowStream {
  pub(super) table: Arc<Table>,
  pub(super) schema: SchemaRef,
  pub(super) transaction: Arc<dyn Transaction>,
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
      .scan(&format!("t{}_r", self.table.id).into_bytes())
      .unwrap();

    if raw_rows.is_empty() {
      self.done = true;
      return Poll::Ready(None);
    }

    let rows = raw_rows
      .iter()
      .map(|(_key, value)| {
        storage::serde::deserialize::<Vec<DataWithValue<Vec<u8>>>>(&value)
          .unwrap()
      })
      .collect::<Vec<Vec<DataWithValue<Vec<u8>>>>>();

    let col_arrays =
      RowConverter::convert_to_columns(&self.table, &self.schema, &rows);

    self.done = true;
    return Poll::Ready(Some(Ok(RecordBatch::try_new(
      self.schema(),
      col_arrays,
    )?)));
  }
}
