use std::any::Any;
use std::sync::Arc;

use datafusion::catalog::schema::SchemaProvider as DfSchemaProvider;
use datafusion::catalog::{
  CatalogList as DfCatalogList, CatalogProvider as DfCatalogProvider,
};
use derive_builder::Builder;

use super::schema::SchemaProviderBuilder;
use crate::storage::Transaction;

pub trait CatalogListProvider: Send + Sync {
  fn get_catalog_list(
    &self,
    transaction: Transaction,
  ) -> Option<Arc<dyn DfCatalogList>>;
}

pub struct SingleCatalogListProvider {
  pub catalog: Arc<str>,
  pub schemas: Arc<Vec<String>>,
}

impl SingleCatalogListProvider {
  pub fn new(catalog: &str, schemas: Arc<Vec<String>>) -> Self {
    Self {
      catalog: catalog.into(),
      schemas,
    }
  }
}

impl CatalogListProvider for SingleCatalogListProvider {
  fn get_catalog_list(
    &self,
    transaction: Transaction,
  ) -> Option<Arc<dyn DfCatalogList>> {
    Some(Arc::new(
      SingleCatalogListBuilder::default()
        .catalog(self.catalog.clone())
        .provider(Arc::new(CatalogProvider {
          catalog: self.catalog.clone(),
          schemas: self.schemas.clone(),
          transaction,
        }))
        .build()
        .unwrap(),
    ))
  }
}

#[derive(Builder)]
pub struct SingleCatalogList {
  catalog: Arc<str>,
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
    vec![self.catalog.to_string()]
  }

  fn catalog(&self, name: &str) -> Option<Arc<dyn DfCatalogProvider>> {
    if *name == *self.catalog {
      return Some(self.provider.clone());
    } else {
      None
    }
  }
}

pub struct CatalogProvider {
  pub catalog: Arc<str>,
  pub schemas: Arc<Vec<String>>,
  pub transaction: Transaction,
}

impl DfCatalogProvider for CatalogProvider {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn schema(&self, schema_name: &str) -> Option<Arc<dyn DfSchemaProvider>> {
    if self.schemas.iter().any(|s| s.as_str() == schema_name) {
      Some(Arc::new(
        SchemaProviderBuilder::default()
          .catalog(self.catalog.clone())
          .schema(schema_name.into())
          .transaction(self.transaction.clone())
          .build()
          .unwrap(),
      ))
    } else {
      None
    }
  }

  fn schema_names(&self) -> Vec<String> {
    self.schemas.as_ref().clone()
  }
}
