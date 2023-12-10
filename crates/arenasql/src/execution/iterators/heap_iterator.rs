use crate::schema::{DataFrame, SerializedCell, Table};
use crate::storage::{KeyValueGroup, StorageHandler};
use crate::{table_rows_prefix_key, Result};

pub(crate) struct HeapIterator<'a> {
  storage: &'a StorageHandler,
  table: &'a Table,
  column_projection: &'a Vec<usize>,
}

impl<'a> HeapIterator<'a> {
  pub fn new(
    storage: &'a StorageHandler,
    table: &'a Table,
    column_projection: &'a Vec<usize>,
  ) -> Self {
    Self {
      storage,
      table,
      column_projection,
    }
  }

  pub fn fill_into(&self, dataframe: &mut DataFrame) -> Result<()> {
    let mut rows_iter = self.storage.kv.scan_with_prefix(
      KeyValueGroup::Rows,
      &table_rows_prefix_key!(self.table.id),
    )?;

    let table_row_prefix = table_rows_prefix_key!(self.table.id);
    while let Some((row_id_with_prefix, row_bytes)) = rows_iter.get() {
      let row_id = &row_id_with_prefix[table_row_prefix.len()..];
      let row = self
        .storage
        .serializer
        .deserialize::<Vec<SerializedCell<&[u8]>>>(row_bytes)?;

      let columns = self
        .column_projection
        .iter()
        .map(|proj| &row[*proj])
        .collect();

      dataframe.append_row(row_id, &columns);
      rows_iter.next();
    }
    Ok(())
  }
}
