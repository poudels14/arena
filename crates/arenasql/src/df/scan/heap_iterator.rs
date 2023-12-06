use crate::schema::Table;
use crate::storage::{KeyValueGroup, RowIterator, StorageOperator};
use crate::{table_rows_prefix_key, Result};

pub(super) struct HeapIterator {}

impl HeapIterator {
  pub fn new(
    table: &Table,
    storage: &StorageOperator,
  ) -> Result<Box<dyn RowIterator>> {
    let iterator = storage.kv.scan_with_prefix(
      KeyValueGroup::Rows,
      &table_rows_prefix_key!(table.id),
    )?;
    Ok(iterator)
  }
}
