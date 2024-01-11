use std::sync::Arc;

use arenasql::arrow::Array;
use arenasql::datafusion::RecordBatch;
use arenasql::pgwire::api::results::{DataRowEncoder, FieldInfo};
use arenasql::pgwire::error::PgWireResult;
use arenasql::pgwire::messages::data::DataRow;
use arenasql::postgres_types::Type;
use arenasql::schema::CTID_COLUMN;

use crate::pgwire::encoder;

pub fn convert_to_rows<'a>(
  schema: &Arc<Vec<FieldInfo>>,
  batch: &'a RecordBatch,
) -> Vec<PgWireResult<DataRow>> {
  let mut encoders = (0..batch.num_rows())
    .map(|_| DataRowEncoder::new(schema.clone()))
    .collect::<Vec<DataRowEncoder>>();

  let column_arrays: Vec<(Arc<dyn Array>, Type)> = schema
    .iter()
    .filter_map(|field| {
      if field.name() == CTID_COLUMN {
        None
      } else {
        batch
          .column_by_name(&field.name())
          .map(|arr| (arr.clone(), field.datatype().to_owned()))
      }
    })
    .collect();

  column_arrays.iter().for_each(|(col_arr, pg_type)| {
    let _ =
      encoder::encode_column_array(encoders.as_mut_slice(), col_arr, pg_type)
        .unwrap();
  });

  encoders
    .into_iter()
    .map(|encoder| encoder.finish())
    .collect::<Vec<PgWireResult<DataRow>>>()
}
