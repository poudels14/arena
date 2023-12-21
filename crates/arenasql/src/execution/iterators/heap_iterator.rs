use crate::schema::{DataFrame, Row, Table};
use crate::storage::{KeyValueGroup, KeyValueIterator, StorageHandler};
use crate::{table_rows_prefix_key, Result};

pub struct HeapIterator<'a> {
  storage: &'a StorageHandler,
  column_projection: &'a Vec<usize>,
  row_prefix: Vec<u8>,
  rows_iter: Box<dyn KeyValueIterator>,
}

impl<'a> HeapIterator<'a> {
  pub fn new(
    storage: &'a StorageHandler,
    table: &'a Table,
    column_projection: &'a Vec<usize>,
  ) -> Self {
    let rows_iter = storage
      .kv
      .scan_with_prefix(KeyValueGroup::Rows, &table_rows_prefix_key!(table.id))
      .unwrap();
    let row_prefix = table_rows_prefix_key!(table.id);
    Self {
      storage,
      column_projection,
      row_prefix,
      rows_iter,
    }
  }

  #[inline]
  pub fn get(&'a self) -> Result<Option<(&'a [u8], Row<'a>)>> {
    if let Some((row_id_with_prefix, row_bytes)) = self.rows_iter.get() {
      let row_id = &row_id_with_prefix[self.row_prefix.len()..];
      let row = self.storage.serializer.deserialize::<Row<'_>>(row_bytes)?;
      Ok(Some((row_id, row)))
    } else {
      Ok(None)
    }
  }

  #[inline]
  pub fn next(&mut self) {
    self.rows_iter.next();
  }

  pub fn fill_into(&mut self, dataframe: &mut DataFrame) -> Result<()> {
    while let Some((row_id, row)) = self.get()? {
      let columns = self
        .column_projection
        .iter()
        .map(|proj| &row[*proj])
        .collect();

      dataframe.append_row(row_id, &columns);
      self.next();
    }
    Ok(())
  }
}
