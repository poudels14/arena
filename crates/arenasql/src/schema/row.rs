use std::sync::Arc;

use datafusion::arrow::array::{
  Array, ArrayRef, BooleanArray, Float32Array, Float64Array, Int32Array,
  Int64Array, StringArray,
};
use datafusion::arrow::datatypes::SchemaRef;
use datafusion::arrow::record_batch::RecordBatch;
use itertools::Itertools;

use super::{DataType, DataWithValue, Table};

pub type RowId = i64;

#[macro_export]
macro_rules! data_with_value {
  ($data:ident, $arr_type:ident, $mapper:expr) => {
    $data
      .as_any()
      .downcast_ref::<$arr_type>()
      .unwrap()
      .iter()
      .map($mapper)
      .collect()
  };
}

pub struct RowConverter {}

impl RowConverter {
  pub fn convert_to_rows<'a>(
    table: &Table,
    batch: &'a RecordBatch,
  ) -> Vec<Vec<DataWithValue<&'a [u8]>>> {
    let row_count = batch.num_rows();
    let col_count = table.columns.len();

    let flat_cols_vec = table
      .columns
      .iter()
      .flat_map(|col| {
        let values = batch
          .column_by_name(&col.name)
          .map(|b| Self::convert_to_data_type_value_vec(&col.data_type, b))
          .unwrap_or(vec![DataWithValue::Null; row_count]);
        values
      })
      .collect::<Vec<DataWithValue<&[u8]>>>();

    let mut flat_rows_vec = Vec::with_capacity(flat_cols_vec.len());

    for ridx in 0..row_count {
      for cidx in 0..col_count {
        flat_rows_vec.push(flat_cols_vec[cidx * row_count + ridx].clone());
      }
    }

    // TODO: return iterator for perf
    flat_rows_vec
      .into_iter()
      .chunks(col_count)
      .into_iter()
      .map(|chunk| chunk.collect())
      .collect::<Vec<Vec<DataWithValue<&'a [u8]>>>>()
  }

  pub fn convert_to_columns(
    table: &Table,
    schema: &SchemaRef,
    rows: &Vec<Vec<DataWithValue<Vec<u8>>>>,
  ) -> Vec<ArrayRef> {
    let selected_col_indices = schema
      .fields
      .iter()
      .map(|f| {
        table
          .columns
          .iter()
          .find(|c| c.name == *f.name())
          .unwrap()
          .id as usize
      })
      .collect::<Vec<usize>>();

    let all_values = rows
      .iter()
      .flatten()
      .collect::<Vec<&DataWithValue<Vec<u8>>>>();

    let row_count = rows.len();
    let col_count = table.columns.len();
    let mut flat_cols_vec =
      Vec::with_capacity(selected_col_indices.len() * row_count);

    for cidx in &selected_col_indices {
      for ridx in 0..row_count {
        flat_cols_vec.push(all_values[ridx * col_count + cidx].clone());
      }
    }

    return flat_cols_vec
      .into_iter()
      .chunks(row_count)
      .into_iter()
      .zip(&selected_col_indices)
      .map(
        |(chunk, col_idx)| match &table.columns[*col_idx].data_type {
          DataType::Text => Arc::new(StringArray::from(
            chunk
              .map(|c| c.as_string())
              .collect::<Vec<Option<String>>>(),
          )) as ArrayRef,
          DataType::Int32 => Arc::new(Int32Array::from(
            chunk.map(|c| c.as_i32()).collect::<Vec<Option<i32>>>(),
          )) as ArrayRef,
          v => unimplemented!("Unsupported value type = {:?}", v),
        },
      )
      .collect::<Vec<ArrayRef>>();
  }

  fn convert_to_data_type_value_vec<'a>(
    data_type: &DataType,
    data: &'a ArrayRef,
  ) -> Vec<DataWithValue<&'a [u8]>> {
    match data_type {
      DataType::Null => (0..data.len())
        .into_iter()
        .map(|_| DataWithValue::Null)
        .collect(),

      DataType::Boolean => {
        data_with_value!(data, BooleanArray, |v| v
          .map(|v| DataWithValue::Boolean(v))
          .unwrap_or_default())
      }
      DataType::Int32 => {
        data_with_value!(data, Int32Array, |v| v
          .map(|v| DataWithValue::Int32(v))
          .unwrap_or_default())
      }
      DataType::Int64 => {
        data_with_value!(data, Int64Array, |v| v
          .map(|v| DataWithValue::Int64(v))
          .unwrap_or_default())
      }
      DataType::Float32 => {
        data_with_value!(data, Float32Array, |v| v
          .map(|v| DataWithValue::Float32(v))
          .unwrap_or_default())
      }
      DataType::Float64 => {
        data_with_value!(data, Float64Array, |v| v
          .map(|v| DataWithValue::Float64(v))
          .unwrap_or_default())
      }
      DataType::Varchar { len: _ } | DataType::Text => {
        data.as_any().downcast_ref::<StringArray>();

        data
          .as_any()
          .downcast_ref::<StringArray>()
          .unwrap()
          .iter()
          .map(|v| {
            v.map(|v| DataWithValue::Blob(v.as_bytes()))
              .unwrap_or_default()
          })
          .collect()
      }
      _ => unimplemented!(),
    }
  }
}
