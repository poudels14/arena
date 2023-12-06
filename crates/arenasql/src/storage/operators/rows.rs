use super::StorageOperator;
use crate::schema::{Row, Table};
use crate::storage::KeyValueGroup;
use crate::{table_rows_prefix_key, Result};

impl StorageOperator {
  pub fn insert_row(
    &self,
    table: &Table,
    row_id: &[u8],
    row: &Row<&[u8]>,
  ) -> Result<()> {
    let row_bytes = self.serializer.serialize(&row)?;
    self.kv.put(
      KeyValueGroup::Rows,
      &vec![table_rows_prefix_key!(table.id).as_slice(), &row_id].concat(),
      &row_bytes,
    )
  }
}
