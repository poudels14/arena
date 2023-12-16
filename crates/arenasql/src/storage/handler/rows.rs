use super::StorageHandler;
use crate::schema::{OwnedRow, Row, Table};
use crate::storage::KeyValueGroup;
use crate::{table_rows_prefix_key, Result};

impl StorageHandler {
  pub fn get_row(
    &self,
    table: &Table,
    row_id: &[u8],
  ) -> Result<Option<OwnedRow>> {
    self
      .kv
      .get(
        KeyValueGroup::Rows,
        &vec![table_rows_prefix_key!(table.id).as_slice(), &row_id].concat(),
      )
      .transpose()
      .map(|bytes| bytes.and_then(|b| self.serializer.deserialize(&b)))
      .transpose()
  }

  pub fn insert_row(
    &self,
    table: &Table,
    row_id: &[u8],
    row: &Row<'_>,
  ) -> Result<()> {
    let row_bytes = self.serializer.serialize(&row)?;
    self.kv.put(
      KeyValueGroup::Rows,
      &vec![table_rows_prefix_key!(table.id).as_slice(), &row_id].concat(),
      &row_bytes,
    )
  }

  pub fn delete_row(&self, table: &Table, row_id: &[u8]) -> Result<()> {
    self.kv.delete(
      KeyValueGroup::Rows,
      &vec![table_rows_prefix_key!(table.id).as_slice(), &row_id].concat(),
    )
  }
}
