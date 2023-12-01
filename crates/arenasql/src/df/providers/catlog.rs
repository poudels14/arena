use std::any::Any;
use std::sync::Arc;

use datafusion::catalog::schema::SchemaProvider as DfSchemaProvider;
use datafusion::catalog::{
  CatalogList as DfCatalogList, CatalogProvider as DfCatalogProvider,
};

use super::schema::SchemaProvider;
use crate::storage::Transaction;

pub trait CatalogListProvider: Send + Sync {
  fn get_catalog_list(
    &self,
    transaction: Transaction,
  ) -> Option<Arc<dyn DfCatalogList>>;
}

pub struct EmptyCatalogListProvider;

impl CatalogListProvider for EmptyCatalogListProvider {
  fn get_catalog_list(
    &self,
    _transaction: Transaction,
  ) -> Option<Arc<dyn DfCatalogList>> {
    None
  }
}

pub struct SingleCatalogListProvider {
  pub catalog: String,
  pub schema: String,
}

impl SingleCatalogListProvider {
  pub fn new(catalog: &str, schema: &str) -> Self {
    Self {
      catalog: catalog.to_string(),
      schema: schema.to_string(),
    }
  }
}

impl CatalogListProvider for SingleCatalogListProvider {
  fn get_catalog_list(
    &self,
    transaction: Transaction,
  ) -> Option<Arc<dyn DfCatalogList>> {
    Some(Arc::new(SingleCatalogList {
      catalog: self.catalog.clone(),
      provider: Arc::new(CatalogProvider {
        schema: self.schema.clone(),
        schema_provider: Arc::new(SchemaProvider {
          catalog: self.catalog.to_owned(),
          schema: self.schema.to_owned(),
          transaction,
        }),
      }),
    }))
  }
}

pub struct SingleCatalogList {
  catalog: String,
  provider: Arc<dyn DfCatalogProvider>,
}

impl DfCatalogList for SingleCatalogList {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn register_catalog(
    &self,
    _name: String,
    _catalog: Arc<dyn DfCatalogProvider>,
  ) -> Option<Arc<dyn DfCatalogProvider>> {
    unimplemented!()
  }

  fn catalog_names(&self) -> Vec<String> {
    vec![self.catalog.clone()]
  }

  fn catalog(&self, name: &str) -> Option<Arc<dyn DfCatalogProvider>> {
    if name == self.catalog {
      return Some(self.provider.clone());
    } else {
      None
    }
  }
}

pub struct CatalogProvider {
  schema: String,
  schema_provider: Arc<dyn DfSchemaProvider>,
}

impl DfCatalogProvider for CatalogProvider {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn schema(&self, schema_name: &str) -> Option<Arc<dyn DfSchemaProvider>> {
    if schema_name == self.schema {
      return Some(self.schema_provider.clone());
    } else {
      None
    }
  }

  fn schema_names(&self) -> Vec<String> {
    vec![self.schema.clone()]
  }
}
