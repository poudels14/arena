use crate::schema::{RowId, Table};
use crate::storage::KeyValueGroup;
use crate::{last_row_id_of_table_key, Result};

use super::StorageOperator;

impl StorageOperator {
  #[inline]
  pub fn generate_next_row_id(&self, table: &Table) -> Result<Vec<u8>> {
    self.kv.atomic_update(
      KeyValueGroup::Locks,
      &last_row_id_of_table_key!(table.id),
      &|old: Option<Vec<u8>>| {
        let new_row_id = old
          .map(|b| RowId::deserialize(&b).add(1))
          .unwrap_or_default();
        Ok(new_row_id.serialize())
      },
    )
  }
}
