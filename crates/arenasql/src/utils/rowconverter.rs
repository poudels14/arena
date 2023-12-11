use datafusion::arrow::record_batch::RecordBatch;
use datafusion::error::Result;

use crate::error::null_constraint_violation;
use crate::schema::{Row, SerializedCell, Table};

// TODO: maybe just return raw bytes instead of SerializedCell?
pub fn convert_to_rows<'a>(
  table: &Table,
  batch: &'a RecordBatch,
) -> Result<Vec<Row<'a>>> {
  let row_count = batch.num_rows();
  let col_count = table.columns.len();

  let mut serialized_col_vecs = table
    .columns
    .iter()
    .map(|col| {
      let values = batch.column_by_name(&col.name).map(|columns_data| {
        let cell =
          SerializedCell::array_ref_to_vec(&table.name, &col, columns_data);
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
    .collect::<Result<Vec<Vec<SerializedCell<'_>>>>>()?;

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
