use std::any::Any;
use std::sync::Arc;

use async_trait::async_trait;
use datafusion::catalog::schema::SchemaProvider as DfSchemaProvider;
use datafusion::datasource::TableProvider;
use datafusion::error::{DataFusionError, Result};

use crate::schema::{Column, Table, TableId};
use crate::storage::Transaction;

pub struct SchemaProvider {
  pub(super) catalog: String,
  pub(super) name: String,
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

  async fn table(&self, name: &str) -> Option<Arc<dyn TableProvider>> {
    let bytes = self.transaction.get(
      format!("m_schema_{}_{}_{}", self.catalog, self.name, name).as_bytes(),
    );

    match bytes {
      Ok(bytes) => {
        bytes.and_then(|b| match bincode::deserialize::<Table>(&b) {
          Ok(schema) => Some(Arc::new(super::table::TableProvider::new(
            schema,
            self.transaction.clone(),
          )) as Arc<dyn TableProvider>),
          Err(e) => {
            tracing::error!("Error deserializing table schema: {:?}", e);
            None
          }
        })
      }
      Err(e) => {
        tracing::error!("Error getting table schema from storage: {:?}", e);
        None
      }
    }
  }

  #[allow(unused_variables)]
  fn register_table(
    &self,
    name: String,
    table: Arc<dyn TableProvider>,
  ) -> Result<Option<Arc<dyn TableProvider>>> {
    let next_table_id = self
      .transaction
      .get_for_update("m_next_table_id".as_bytes(), true)
      .map_err(|e| DataFusionError::Execution(e.to_string()))?
      .map(|bytes| bincode::deserialize::<TableId>(&bytes).unwrap())
      .unwrap_or(1);

    let columns = table
      .schema()
      .fields
      .iter()
      .enumerate()
      .map(|(idx, field)| Column::from_field(idx as u16, field))
      .collect();

    let table = Table {
      id: next_table_id,
      name: name.to_owned(),
      columns,
      constraints: vec![],
    };

    bincode::serialize::<Table>(&table)
      .map_err(|e| {
        DataFusionError::Execution("Error serializing table schema".to_owned())
      })
      .and_then(|value| {
        self
          .transaction
          .put_all(
            vec![
              (
                format!("m_schema_{}_{}_{}", self.catalog, self.name, name)
                  .as_bytes(),
                value.as_ref(),
              ),
              (
                "m_next_table_id".as_bytes(),
                bincode::serialize(&(next_table_id + 1)).unwrap().as_ref(),
              ),
            ]
            .as_slice(),
          )
          .map_err(|e| DataFusionError::Execution(e.to_string()))
      })?;

    Ok(Some(Arc::new(super::table::TableProvider::new(
      table,
      self.transaction.clone(),
    )) as Arc<dyn TableProvider>))
  }

  fn table_exist(&self, _name: &str) -> bool {
    unimplemented!()
  }
}
