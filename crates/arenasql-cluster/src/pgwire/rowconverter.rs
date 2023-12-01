use std::sync::Arc;

use arenasql::records::RecordBatch;
use pgwire::api::results::{DataRowEncoder, FieldInfo};
use pgwire::error::PgWireResult;
use pgwire::messages::data::DataRow;

use super::encoder::ColumnEncoder;
use crate::pgwire::datatype;

pub fn convert_to_rows<'a>(
  batch: &'a RecordBatch,
) -> Vec<PgWireResult<DataRow>> {
  let batch_fields = &batch.schema().fields;
  let fields: Vec<FieldInfo> = batch_fields
    .iter()
    .map(|field| datatype::to_field_info(field))
    .collect();
  let schema = Arc::new(fields);

  let mut encoders = (0..batch.num_rows())
    .map(|_| DataRowEncoder::new(schema.clone()))
    .collect::<Vec<DataRowEncoder>>();

  let column_arrays: Vec<Box<dyn ColumnEncoder>> = batch_fields
    .iter()
    .map(|field| {
      batch
        .column_by_name(&field.name())
        .map(|arr| Box::new(arr) as Box<dyn ColumnEncoder>)
        .unwrap()
    })
    .collect();

  column_arrays.iter().for_each(|col_arr| {
    col_arr
      .encode_column_array(encoders.as_mut_slice())
      .unwrap();
  });

  encoders
    .into_iter()
    .map(|encoder| encoder.finish())
    .collect::<Vec<PgWireResult<DataRow>>>()
}
