use std::sync::Arc;

use datafusion::arrow::datatypes::SchemaRef;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::error::Result;
use derivative::Derivative;

use super::filter::Filter;
use super::unique_index_iterator;
use crate::df::scan::heap_iterator::HeapIterator;
use crate::schema::{ColumnArrayBuilder, SerializedCell, Table};
use crate::storage::Transaction;

#[allow(dead_code)]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RowsStream {
  pub(super) table: Arc<Table>,
  pub(crate) projection: Vec<usize>,
  pub(super) schema: SchemaRef,
  pub(super) filters: Vec<Filter>,
  #[derivative(Debug = "ignore")]
  pub(super) transaction: Transaction,
  pub(super) done: bool,
}

impl RowsStream {
  fn schema(&self) -> SchemaRef {
    self.schema.clone()
  }
}

#[allow(dead_code)]
impl RowsStream {
  pub async fn scan_table(&mut self) -> Result<RecordBatch> {
    let transaction = self.transaction.lock()?;

    let mut column_list_builders: Vec<ColumnArrayBuilder> = self
      .projection
      .iter()
      .map(|idx| {
        // TODO(sagar): pass in limit and use it as capacity when possible
        ColumnArrayBuilder::from(&self.table.columns[*idx].data_type, 5_000)
      })
      .collect();

    let index_with_lowest_cost =
      Filter::find_index_with_lowest_cost(&self.table.indexes, &self.filters);

    let mut rows_iterator = if let Some(index) = index_with_lowest_cost {
      unique_index_iterator::new(
        &self.table,
        index,
        &self.filters,
        &transaction,
      )?
    } else {
      HeapIterator::new(&self.table, &transaction)?
    };

    // TODO: try if sending rows in batches improves perf
    while let Some((_key, value)) = rows_iterator.get() {
      let row = self
        .transaction
        .serializer
        .deserialize::<Vec<SerializedCell<&[u8]>>>(value)?;

      self
        .projection
        .iter()
        .enumerate()
        .for_each(|(builder_idx, col_idx)| {
          column_list_builders[builder_idx]
            .append(unsafe { row.get_unchecked(*col_idx) });
        });
      rows_iterator.next();
    }

    let col_arrays = column_list_builders
      .into_iter()
      .map(|b| b.finish())
      .collect();

    drop(rows_iterator);
    drop(transaction);
    self.done = true;
    Ok(RecordBatch::try_new(self.schema(), col_arrays)?)
  }
}
