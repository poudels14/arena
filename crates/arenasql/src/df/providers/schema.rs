use std::any::Any;
use std::sync::Arc;

use async_trait::async_trait;
use datafusion::catalog::schema::SchemaProvider as DfSchemaProvider;
use datafusion::datasource::TableProvider as DfTableProvider;
use datafusion::error::Result;

use crate::schema::{Column, Table, TableId};
use crate::storage::{Serializer, Transaction};
use crate::{df_execution_error, next_table_id_key, table_schema_key};

use super::table::TableProvider;

pub struct SchemaProvider {
  pub(super) catalog: String,
  pub(super) schema: String,
  pub(super) transaction: Arc<dyn Transaction>,
}

#[async_trait]
impl DfSchemaProvider for SchemaProvider {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn table_names(&self) -> Vec<String> {
    unimplemented!()
  }

  async fn table(&self, name: &str) -> Option<Arc<dyn DfTableProvider>> {
    self
      .transaction
      .get_or_log_error(table_schema_key!(self.catalog, self.schema, name))
      .and_then(|bytes| {
        Serializer::FixedInt
          .deserialize_or_log_error(&bytes)
          .map(|table| {
            Arc::new(TableProvider::new(table, self.transaction.clone()))
              as Arc<dyn DfTableProvider>
          })
      })
  }

  #[allow(unused_variables)]
  fn register_table(
    &self,
    name: String,
    table: Arc<dyn DfTableProvider>,
  ) -> Result<Option<Arc<dyn DfTableProvider>>> {
    let serializer = Serializer::FixedInt;
    let new_table_id = self
      .transaction
      .get_for_update(next_table_id_key!(), true)
      .map_err(|e| df_execution_error!("Storage error: {}", e.to_string()))?
      .map(|bytes| {
        serializer.deserialize::<TableId>(&bytes).map_err(|e| {
          df_execution_error!("Serialization error: {}", e.to_string())
        })
      })
      .unwrap_or(Ok(1))?;

    let columns = table
      .schema()
      .fields
      .iter()
      .enumerate()
      .map(|(idx, field)| Column::from_field(idx as u16, field))
      .collect();

    let table = Table {
      id: new_table_id,
      name: name.to_owned(),
      columns,
      constraints: vec![],
    };

    serializer
      .serialize::<Table>(&table)
      .and_then(|table| {
        serializer
          .serialize(&(new_table_id + 1))
          .map(|id| (id, table))
      })
      .map_err(|e| {
        df_execution_error!("Serialization error: {}", e.to_string())
      })
      .and_then(|(next_table_id, table_bytes)| {
        self
          .transaction
          .put_all(
            vec![
              (next_table_id_key!(), next_table_id.as_slice()),
              (
                table_schema_key!(self.catalog, self.schema, name),
                table_bytes.as_slice(),
              ),
            ]
            .as_slice(),
          )
          .map_err(|e| df_execution_error!("Storage error: {}", e.to_string()))
      })?;

    Ok(Some(
      Arc::new(TableProvider::new(table, self.transaction.clone()))
        as Arc<dyn DfTableProvider>,
    ))
  }

  fn table_exist(&self, _name: &str) -> bool {
    unimplemented!()
  }
}
