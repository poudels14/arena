use super::StorageHandler;
use crate::schema::{Table, TableId};
use crate::storage::{KeyValueGroup, Serializer};
use crate::{
  last_table_id_key, table_schema_key, table_schemas_prefix_key, Result,
};

impl StorageHandler {
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

  pub fn get_table_schema(
    &self,
    catalog: &str,
    schema: &str,
    table: &str,
  ) -> Result<Option<Table>> {
    Ok(
      self
        .kv
        .get(
          KeyValueGroup::Schemas,
          table_schema_key!(catalog, schema, table),
        )?
        .and_then(|bytes| {
          Serializer::FixedInt.deserialize_or_log_error::<Table>(&bytes)
        }),
    )
  }

  pub fn get_all_table_schemas(
    &self,
    catalog: &str,
    schema: &str,
  ) -> Result<Vec<Table>> {
    let mut iter = self.kv.scan_with_prefix(
      KeyValueGroup::Schemas,
      table_schemas_prefix_key!(catalog, schema),
    )?;

    let mut tables = Vec::new();
    while let Some((_key, value)) = iter.get() {
      let table = Serializer::FixedInt.deserialize::<Table>(&value)?;
      tables.push(table);
      iter.next();
    }
    Ok(tables)
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
}
