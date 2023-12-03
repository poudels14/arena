use std::any::Any;
use std::sync::Arc;

use async_trait::async_trait;
use datafusion::catalog::schema::SchemaProvider as DfSchemaProvider;
use datafusion::datasource::TableProvider as DfTableProvider;
use datafusion::error::Result;

use super::table::TableProvider;
use crate::schema::Table;
use crate::storage::Transaction;

pub struct SchemaProvider {
  pub(super) catalog: String,
  pub(super) schema: String,
  pub(super) transaction: Transaction,
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
    let storage_handler = self.transaction.lock().ok()?;
    storage_handler
      .get_table_schema(&self.catalog, &self.schema, name)
      .map(|table| {
        Arc::new(TableProvider::new(table, self.transaction.clone()))
          as Arc<dyn DfTableProvider>
      })
  }

  #[allow(unused_variables)]
  fn register_table(
    &self,
    name: String,
    table: Arc<dyn DfTableProvider>,
  ) -> Result<Option<Arc<dyn DfTableProvider>>> {
    let storage_handler = self.transaction.lock()?;

    let new_table_id = storage_handler.get_next_table_id()?;
    let table = Table::new(new_table_id, &name, table)?;

    storage_handler.put_table_schema(&self.catalog, &self.schema, &table)?;
    Ok(Some(
      Arc::new(TableProvider::new(table, self.transaction.clone()))
        as Arc<dyn DfTableProvider>,
    ))
  }

  fn table_exist(&self, name: &str) -> bool {
    self
      .transaction
      .lock()
      .map(|txn| txn.has_table(&self.catalog, &self.schema, name))
      .unwrap_or(false)
  }
}
