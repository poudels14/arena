use super::StorageOperator;
use crate::schema::{Table, TableId};
use crate::storage::RawIterator;
use crate::storage::{KeyValueGroup, Serializer};
use crate::{
  last_table_id_key, table_rows_prefix_key, table_schema_key, Result,
};

impl StorageOperator {
  #[inline]
  pub fn get_next_table_id(&self) -> Result<TableId> {
    let serializer = Serializer::FixedInt;
    self
      .kv
      .atomic_update(
        KeyValueGroup::Locks,
        last_table_id_key!(),
        &|prev: Option<Vec<u8>>| {
          let last_table_id = prev
            .map(|bytes| serializer.deserialize::<TableId>(&bytes))
            // Return 0 if there's no value in `last_table_id_key`
            .unwrap_or(Ok(0));
          last_table_id
            .and_then(|id| Ok(serializer.serialize::<TableId>(&(id + 1))?))
        },
      )
      .and_then(|id_bytes| serializer.deserialize::<TableId>(&id_bytes))
  }

  pub fn has_table(&self, catalog: &str, schema: &str, table: &str) -> bool {
    self
      .get_or_log_error(
        KeyValueGroup::Schemas,
        table_schema_key!(catalog, schema, table),
      )
      .is_some()
  }

  pub fn get_table_schema(
    &self,
    catalog: &str,
    schema: &str,
    table: &str,
  ) -> Option<Table> {
    self
      .get_or_log_error(
        KeyValueGroup::Schemas,
        table_schema_key!(catalog, schema, table),
      )
      .and_then(|bytes| {
        Serializer::FixedInt.deserialize_or_log_error::<Table>(&bytes)
      })
  }

  pub fn put_table_schema(
    &self,
    catalog: &str,
    schema: &str,
    table: &Table,
  ) -> Result<()> {
    let table_bytes = Serializer::FixedInt.serialize::<Table>(&table)?;
    self.kv.put(
      KeyValueGroup::Schemas,
      table_schema_key!(catalog, schema, &table.name),
      &table_bytes,
    )
  }

  pub fn scan_raw(&self, table: &Table) -> Result<Box<dyn RawIterator>> {
    self
      .kv
      .scan_raw(KeyValueGroup::Rows, &table_rows_prefix_key!(table.id))
  }
}
