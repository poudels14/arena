use std::sync::Arc;

use datafusion::arrow::array::{ArrayRef, Int32Array, StringArray};
use datafusion::arrow::datatypes::SchemaRef;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::error::Result;
use itertools::Itertools;

use crate::error::null_constraint_violation;

use super::{DataType, SerializedCell, Table};

pub type RowId = i64;

pub struct RowConverter {}

impl RowConverter {
  // TODO: maybe just return raw bytes instead of SerializedCell?
  pub fn convert_to_rows<'a>(
    table: &Table,
    batch: &'a RecordBatch,
  ) -> Result<Vec<Vec<SerializedCell<&'a [u8]>>>> {
    let row_count = batch.num_rows();
    let col_count = table.columns.len();

    let mut serialized_col_vecs = table
      .columns
      .iter()
      .map(|col| {
        let values = batch.column_by_name(&col.name).map(|columns_data| {
          let cell =
            SerializedCell::array_ref_to_vec(&table, &col, columns_data);
          cell
        });
        match values {
          Some(arr) => arr,
          None => {
            if !col.nullable {
              return Err(null_constraint_violation(&table.name, &col.name));
            } else {
              Ok(
                (0..row_count)
                  .into_iter()
                  .map(|_| SerializedCell::Null)
                  .collect(),
              )
            }
          }
        }
      })
      .collect::<Result<Vec<Vec<SerializedCell<&[u8]>>>>>()?;

    // Convert col * row array to row * column
    let mut flat_rows_vec = Vec::with_capacity(row_count);
    for ridx in 0..row_count {
      let mut row = Vec::with_capacity(col_count);
      for cidx in 0..col_count {
        row.push(std::mem::take(&mut serialized_col_vecs[cidx][ridx]))
      }
      flat_rows_vec.push(row);
    }

    // TODO: return iterator for perf
    Ok(flat_rows_vec)
  }

  pub fn convert_to_columns(
    table: &Table,
    schema: &SchemaRef,
    rows: &Vec<Vec<SerializedCell<Vec<u8>>>>,
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
      .collect::<Vec<&SerializedCell<Vec<u8>>>>();

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
}
