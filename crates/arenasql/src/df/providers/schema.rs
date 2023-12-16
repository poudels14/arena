use std::any::Any;
use std::sync::Arc;

use async_trait::async_trait;
use datafusion::catalog::schema::SchemaProvider as DfSchemaProvider;
use datafusion::datasource::TableProvider as DfTableProvider;
use datafusion::error::Result;
use derive_builder::Builder;
use tokio::runtime::Handle;

use super::table::TableProvider;
use crate::schema::{IndexType, Table, TableIndex};
use crate::storage::Transaction;

#[derive(Builder)]
pub struct SchemaProvider {
  transaction: Transaction,
}

#[async_trait]
impl DfSchemaProvider for SchemaProvider {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn table_names(&self) -> Vec<String> {
    self.transaction.state().table_names()
  }

  // Note: for each insert, this table gets called twice
  async fn table(&self, name: &str) -> Option<Arc<dyn DfTableProvider>> {
    let table = self.transaction.state().get_table(name)?;
    Some(
      Arc::new(TableProvider::new(table, self.transaction.clone()))
        as Arc<dyn DfTableProvider>,
    )
  }

  #[allow(unused_variables)]
  fn register_table(
    &self,
    name: String,
    table: Arc<dyn DfTableProvider>,
  ) -> Result<Option<Arc<dyn DfTableProvider>>> {
    let storage_handler = self.transaction.lock(true)?;
    let new_table_id = storage_handler.get_next_table_id()?;

    let mut table = Table::new(new_table_id, &name, table)?;
    let constraints = table.constraints.clone();
    constraints
      .map(|constraints| {
        constraints
          .iter()
          .filter(|constraint| constraint.needs_index())
          .map(|constraint| {
            let index_id = storage_handler.get_next_table_index_id()?;
            table.add_index(
              index_id,
              IndexType::from_constraint(constraint),
              None,
            )
          })
          .collect::<crate::Result<Vec<TableIndex>>>()
      })
      .transpose()?;

    let state = &self.transaction.state();
    let table = Arc::new(table);
    let mut schema_lock = tokio::task::block_in_place(|| {
      Handle::current()
        .block_on(async { state.acquire_table_schema_write_lock(&name).await })
    })?;

    storage_handler.put_table_schema(
      &state.catalog(),
      &state.schema(),
      &table,
    )?;

    schema_lock.table = Some(table.clone());
    state.hold_table_schema_lock(schema_lock)?;

    Ok(Some(
      Arc::new(TableProvider::new(table, self.transaction.clone()))
        as Arc<dyn DfTableProvider>,
    ))
  }

  fn table_exist(&self, name: &str) -> bool {
    self.transaction.state().get_table(name).is_some()
  }
}
