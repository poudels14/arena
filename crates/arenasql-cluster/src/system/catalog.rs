use std::any::Any;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use arenasql::storage::Transaction;
use arenasql::{
  CatalogListProvider, CatalogProvider, DatafusionCatalogList,
  DatafusionCatalogProvider,
};
use derive_builder::Builder;

#[derive(Clone, Debug, Builder)]
pub struct CatalogListOptions {
  cluster_dir: Arc<PathBuf>,
  /// List of schemas visible to the transaction
  schemas: Arc<Vec<String>>,
}

pub struct ArenaClusterCatalogListProvider {
  options: CatalogListOptions,
}

impl ArenaClusterCatalogListProvider {
  pub fn with_options(options: CatalogListOptions) -> Self {
    Self { options }
  }
}

impl CatalogListProvider for ArenaClusterCatalogListProvider {
  fn get_catalog_list(
    &self,
    transaction: Transaction,
  ) -> Option<Arc<dyn DatafusionCatalogList>> {
    Some(Arc::new(DirectoryCatalogList {
      transaction,
      options: self.options.clone(),
    }))
  }
}

pub struct DirectoryCatalogList {
  options: CatalogListOptions,
  transaction: Transaction,
}

impl DirectoryCatalogList {
  fn get_catalog_dir(&self, name: &str) -> PathBuf {
    self.options.cluster_dir.join("catalogs").join(&name)
  }
}

impl DatafusionCatalogList for DirectoryCatalogList {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn register_catalog(
    &self,
    name: String,
    _catalog: Arc<dyn DatafusionCatalogProvider>,
  ) -> Option<Arc<dyn DatafusionCatalogProvider>> {
    match self.catalog(&name) {
      Some(catalog) => Some(catalog),
      None => {
        let catalog_dir = self.get_catalog_dir(&name);
        std::fs::create_dir(&catalog_dir)
          .with_context(|| {
            format!(
              "Failed to create new catalog's directory: {:?}",
              catalog_dir
            )
          })
          .unwrap();

        self.catalog(&name)
      }
    }
  }

  fn catalog_names(&self) -> Vec<String> {
    panic!("Listing catalog names not supported");
  }

  fn catalog(&self, name: &str) -> Option<Arc<dyn DatafusionCatalogProvider>> {
    if self.get_catalog_dir(&name).exists() {
      Some(Arc::new(CatalogProvider {
        catalog: name.into(),
        schemas: self.options.schemas.clone(),
        transaction: self.transaction.clone(),
      }))
    } else {
      None
    }
  }
}
