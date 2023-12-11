use datafusion::arrow::array::{ArrayBuilder, BinaryBuilder};
use datafusion::arrow::datatypes::SchemaRef;
use datafusion::arrow::record_batch::RecordBatch;

use super::{ColumnArrayBuilder, DataType, SerializedCell};
use crate::{Error, Result};

pub struct DataFrame {
  row_ids: BinaryBuilder,
  column_builders: Vec<ColumnArrayBuilder>,
}

impl DataFrame {
  pub fn with_capacity(
    row_capacity: usize,
    columns: Vec<(String, DataType)>,
  ) -> Self {
    let column_builders: Vec<ColumnArrayBuilder> = columns
      .iter()
      .map(|col| {
        // TODO(sagar): pass in limit and use it as capacity when possible
        ColumnArrayBuilder::from(&col.1, 200)
      })
      .collect();

    Self {
      row_ids: BinaryBuilder::with_capacity(
        row_capacity,
        /* u64 = 8bytes */
        8,
      ),
      column_builders,
    }
  }

  #[inline]
  pub fn append_row(
    &mut self,
    row_id: &[u8],
    columns: &Vec<&SerializedCell<'_>>,
  ) {
    self.row_ids.append_value(row_id);
    columns
      .iter()
      .enumerate()
      .for_each(|(i, cell)| self.column_builders[i].append(cell))
  }

  pub fn row_count(&self) -> usize {
    self.row_ids.len()
  }

  pub fn to_record_batch(self, schema: SchemaRef) -> Result<RecordBatch> {
    let col_arrays = self
      .column_builders
      .into_iter()
      .map(|b| b.finish())
      .collect();
    Ok(
      RecordBatch::try_new(schema, col_arrays)
        .map_err(|e| Error::DataFusionError(e.to_string()))?,
    )
  }
}
