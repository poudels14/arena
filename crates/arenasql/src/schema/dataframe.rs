use std::sync::Arc;

use datafusion::arrow::array::{ArrayRef, UInt64Builder};
use datafusion::arrow::datatypes::SchemaRef;
use datafusion::arrow::record_batch::{RecordBatch, RecordBatchOptions};
use datafusion::error::Result;

use super::column::CTID_COLUMN;
use super::{ColumnArrayBuilder, DataType, RowId, SerializedCell};

pub struct DataFrame {
  row_ids: UInt64Builder,
  arrays: Option<Vec<ArrayRef>>,
  column_builders: Vec<ColumnArrayBuilder>,
  include_row_id: bool,
  row_count: usize,
}

impl DataFrame {
  pub fn empty() -> Self {
    Self {
      row_ids: UInt64Builder::with_capacity(0),
      arrays: None,
      column_builders: vec![],
      include_row_id: false,
      row_count: 0,
    }
  }

  pub fn from_arrays(arrays: Vec<ArrayRef>) -> Self {
    Self {
      row_ids: UInt64Builder::with_capacity(0),
      arrays: Some(arrays),
      column_builders: vec![],
      include_row_id: false,
      row_count: 0,
    }
  }

  pub fn with_capacity(
    row_capacity: usize,
    columns: Vec<(
      // column name
      String,
      DataType,
    )>,
  ) -> Self {
    let mut include_row_id = false;
    let column_builders: Vec<ColumnArrayBuilder> = columns
      .iter()
      .filter_map(|col| {
        if col.0 == CTID_COLUMN {
          include_row_id = true;
          None
        } else {
          Some(ColumnArrayBuilder::from(&col.1, row_capacity))
        }
      })
      .collect();

    Self {
      row_ids: UInt64Builder::with_capacity(row_capacity),
      arrays: None,
      column_builders,
      include_row_id,
      row_count: 0,
    }
  }

  #[inline]
  pub fn append_row(
    &mut self,
    // row id bytes
    row_id: &[u8],
    columns: &Vec<&SerializedCell<'_>>,
  ) {
    columns
      .iter()
      .enumerate()
      .for_each(|(i, cell)| self.column_builders[i].append(cell));
    self.row_ids.append_value(RowId::deserialize(&row_id).0);
    self.row_count += 1;
  }

  pub fn row_count(&self) -> usize {
    self
      .arrays
      .as_ref()
      .and_then(|arr| arr.get(0).map(|c| c.len()))
      .unwrap_or(self.row_count)
  }

  pub fn to_record_batch(mut self, schema: SchemaRef) -> Result<RecordBatch> {
    let row_count = self.row_count();
    let col_arrays = self.arrays.unwrap_or_else(|| {
      self
        .column_builders
        .into_iter()
        .map(|b| b.finish())
        .chain(if self.include_row_id {
          vec![Arc::new(self.row_ids.finish()) as ArrayRef]
        } else {
          vec![]
        })
        .collect()
    });

    let mut batch_options = RecordBatchOptions::default();
    batch_options = batch_options.with_row_count(Some(row_count));

    Ok(RecordBatch::try_new_with_options(
      schema,
      col_arrays,
      &batch_options,
    )?)
  }
}
