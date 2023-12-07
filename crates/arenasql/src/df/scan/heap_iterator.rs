use crate::schema::Table;
use crate::storage::{KeyValueGroup, RowIterator, StorageHandler};
use crate::{table_rows_prefix_key, Result};

pub(super) struct HeapIterator {}

impl HeapIterator {
  pub fn new(
    table: &Table,
    storage: &StorageHandler,
  ) -> Result<Box<dyn RowIterator>> {
    let iterator = storage.kv.scan_with_prefix(
      KeyValueGroup::Rows,
      &table_rows_prefix_key!(table.id),
    )?;
    Ok(iterator)
  }
}
