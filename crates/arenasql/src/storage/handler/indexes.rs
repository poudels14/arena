use super::StorageHandler;
use crate::schema::{Row, SerializedCell, Table, TableIndexId};
use crate::storage::{KeyValueGroup, Serializer};
use crate::{index_row_key, last_table_index_id_key, Error, Result};

impl StorageHandler {
  /// Table index id is unique to the database
  #[inline]
  pub fn get_next_table_index_id(&self) -> Result<TableIndexId> {
    let serializer = Serializer::FixedInt;
    self
      .kv
      .atomic_update(
        KeyValueGroup::Locks,
        last_table_index_id_key!(),
        &|prev: Option<Vec<u8>>| {
          let last_index_id = prev
            .map(|bytes| serializer.deserialize::<TableIndexId>(&bytes))
            // Return 0 if there's no value in `last_table_index_id_key`
            .unwrap_or(Ok(0));
          last_index_id
            .and_then(|id| Ok(serializer.serialize::<TableIndexId>(&(id + 1))?))
        },
      )
      .and_then(|id_bytes| serializer.deserialize::<TableIndexId>(&id_bytes))
  }

  /// Adds the row to all the indexes and returns error if
  /// any of the index constraints is violated
  pub fn add_row_to_indexes(
    &self,
    table: &Table,
    row_id_bytes: &[u8],
    row: &Row<&[u8]>,
  ) -> Result<()> {
    for table_index in table.indexes.iter() {
      let projected_cells = row.project(&table_index.columns);
      let serialized_projected_row =
        self
          .serializer
          .serialize::<Vec<&SerializedCell<&[u8]>>>(&projected_cells)?;
      let index_key = index_row_key!(table_index.id, &serialized_projected_row);
      if !table_index.allow_duplicates {
        if self.kv.get(KeyValueGroup::Indexes, &index_key)?.is_some() {
          return Err(Error::UniqueConstaintViolated {
            data: projected_cells.iter().map(|c| (*c).into()).collect(),
            columns: table.project_columns(&table_index.columns),
            constraint: table_index.name.clone(),
          });
        }

        self
          .kv
          .put(KeyValueGroup::Indexes, &index_key, row_id_bytes)?;
      } else {
        // If index allows duplicates, add row_id to the key-value key
        unimplemented!()
      }
    }
    Ok(())
  }
}
