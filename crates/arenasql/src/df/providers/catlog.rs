use std::any::Any;
use std::sync::Arc;

use dashmap::DashMap;
use datafusion::catalog::schema::SchemaProvider as DfSchemaProvider;
use datafusion::catalog::{
  CatalogList as DfCatalogList, CatalogProvider as DfCatalogProvider,
};

use crate::runtime::RuntimeEnv;
use crate::storage::Transaction;

#[derive(Clone)]
pub struct CatalogList {
  pub runtime: RuntimeEnv,
  pub catlogs: DashMap<String, Arc<dyn Transaction>>,
}

impl DfCatalogList for CatalogList {
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
    unimplemented!();
  }

  fn catalog(&self, name: &str) -> Option<Arc<dyn DfCatalogProvider>> {
    return self.catlogs.get(name).map(|transaction| {
      Arc::new(CatalogProvider {
        name: name.to_owned(),
        runtime: self.runtime.clone(),
        transaction: transaction.clone(),
      }) as Arc<dyn DfCatalogProvider>
    });
  }
}

pub struct CatalogProvider {
  name: String,
  #[allow(unused)]
  runtime: RuntimeEnv,
  transaction: Arc<dyn Transaction>,
}

impl DfCatalogProvider for CatalogProvider {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn schema(&self, name: &str) -> Option<Arc<dyn DfSchemaProvider>> {
    Some(Arc::new(super::schema::SchemaProvider {
      catalog: self.name.to_owned(),
      name: name.to_owned(),
      transaction: self.transaction.clone(),
    }))
  }

  fn schema_names(&self) -> Vec<String> {
    unimplemented!();
  }
}
