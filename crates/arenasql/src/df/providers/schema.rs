use std::any::Any;
use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use datafusion::catalog::schema::SchemaProvider as DfSchemaProvider;
use datafusion::datasource::TableProvider as DfTableProvider;
use datafusion::error::Result;

use super::table::TableProvider;
use crate::schema::Table;
use crate::storage::Transaction;

pub struct SchemaProvider {
  transaction: Transaction,
  new_tables: DashMap<String, Arc<Table>>,
}

impl SchemaProvider {
  pub fn new(transaction: Transaction) -> Self {
    Self {
      transaction,
      new_tables: DashMap::new(),
    }
  }
}

#[async_trait]
impl DfSchemaProvider for SchemaProvider {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn table_names(&self) -> Vec<String> {
    unimplemented!()
  }

  // Note: for each insert, this table gets called twice
  async fn table(&self, name: &str) -> Option<Arc<dyn DfTableProvider>> {
    // Note: need to look for the table in the `new_tables` map first to
    // prevent deadlock that happens when `schema_factory.get_table` is called
    // from the same transaction that holds the table lock
    let table = self
      .new_tables
      .get(name)
      .map(|kv| kv.value().clone())
      .or_else(|| self.transaction.schema_factory.get_table(name))?;
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
    self.transaction.acquire_table_lock(&name)?;

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
            table.add_index(index_id, constraint)
          })
          .collect::<crate::Result<Vec<()>>>()
      })
      .transpose()?;

    let schema_factory = &self.transaction.schema_factory;
    storage_handler.put_table_schema(
      &schema_factory.catalog,
      &schema_factory.schema,
      &table,
    )?;

    let table = Arc::new(table);
    self.new_tables.insert(table.name.to_owned(), table.clone());

    Ok(Some(
      Arc::new(TableProvider::new(table, self.transaction.clone()))
        as Arc<dyn DfTableProvider>,
    ))
  }

  fn table_exist(&self, name: &str) -> bool {
    // Note: check the new_table first to avoid deadlock
    self.new_tables.contains_key(name)
      || self.transaction.schema_factory.get_table(name).is_some()
  }
}
