use std::sync::Arc;

use datafusion::arrow::array::{ArrayBuilder, ArrayRef, Int64Builder};
use datafusion::arrow::datatypes::{
  DataType as DfDataType, Field, Schema, SchemaRef,
};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::error::DataFusionError;

use super::{ColumnArrayBuilder, DataType, SerializedCell};
use crate::storage::Serializer;
use crate::{Error, Result};

pub struct DataFrame {
  row_ids: Int64Builder,
  column_builders: Vec<ColumnArrayBuilder>,
}

impl DataFrame {
  pub fn with_capacity(
    row_capacity: usize,
    columns: Vec<(
      // column name
      String,
      DataType,
    )>,
  ) -> Self {
    let column_builders: Vec<ColumnArrayBuilder> = columns
      .iter()
      .map(|col| ColumnArrayBuilder::from(&col.1, row_capacity))
      .collect();

    Self {
      row_ids: Int64Builder::with_capacity(row_capacity),
      column_builders,
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

    self
      .row_ids
      .append_value(Serializer::FixedInt.deserialize(row_id).unwrap());
  }

  pub fn row_count(&self) -> usize {
    self.row_ids.len()
  }

  pub fn to_record_batch(mut self, schema: SchemaRef) -> Result<RecordBatch> {
    let col_arrays = self
      .column_builders
      .into_iter()
      .map(|b| b.finish())
      .chain(vec![Arc::new(self.row_ids.finish()) as ArrayRef])
      .collect();
    let schema_with_virtual_cols = Schema::new(
      schema
        .fields()
        .iter()
        .map(|f| f.clone())
        .chain(
          vec![Arc::new(Field::new("ctid", DfDataType::Int64, false))]
            .into_iter(),
        )
        .collect::<Vec<Arc<Field>>>(),
    )
    .into();

    Ok(
      RecordBatch::try_new(schema_with_virtual_cols, col_arrays).map_err(
        |e| Error::DataFusionError(DataFusionError::ArrowError(e).into()),
      )?,
    )
  }
}
