use std::any::Any;
use std::sync::Arc;

use async_trait::async_trait;
use datafusion::catalog::schema::SchemaProvider as DfSchemaProvider;
use datafusion::datasource::TableProvider as DfTableProvider;
use datafusion::error::Result;
use tokio::runtime::Handle;

use super::table::TableProvider;
use crate::schema::{IndexType, Table};
use crate::storage::Transaction;

pub struct SchemaProvider {
  transaction: Transaction,
}

impl SchemaProvider {
  pub fn new(transaction: Transaction) -> Self {
    Self { transaction }
  }
}

#[async_trait]
impl DfSchemaProvider for SchemaProvider {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn table_names(&self) -> Vec<String> {
    self.transaction.table_names()
  }

  // Note: for each insert, this table gets called twice
  async fn table(&self, name: &str) -> Option<Arc<dyn DfTableProvider>> {
    let table = self.transaction.get_table(name)?;
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
    let schema_factory = &self.transaction.schema_factory;
    let mut table_lock = tokio::task::block_in_place(|| {
      Handle::current().block_on(async {
        self
          .transaction
          .acquire_table_schema_write_lock(&name)
          .await
      })
    })?;

    let storage_handler = self.transaction.lock()?;
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
          .collect::<crate::Result<Vec<()>>>()
      })
      .transpose()?;

    storage_handler.put_table_schema(
      &schema_factory.catalog,
      &schema_factory.schema,
      &table,
    )?;

    let table = Arc::new(table);
    table_lock.table = Some(table.clone());
    self.transaction.hold_table_write_lock(table_lock)?;

    Ok(Some(
      Arc::new(TableProvider::new(table, self.transaction.clone()))
        as Arc<dyn DfTableProvider>,
    ))
  }

  fn table_exist(&self, name: &str) -> bool {
    self.transaction.get_table(name).is_some()
  }
}
