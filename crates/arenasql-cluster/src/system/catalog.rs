use std::any::Any;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use arenasql::datafusion::{DatafusionCatalogList, DatafusionCatalogProvider};
use arenasql::execution::TransactionHandle;
use arenasql::{CatalogListProvider, CatalogProvider};
use derive_builder::Builder;

#[derive(Clone, Debug, Builder)]
pub struct CatalogListOptions {
  cluster_dir: Arc<PathBuf>,
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
    _catalog: Arc<str>,
    schemas: Arc<Vec<String>>,
    transaction: TransactionHandle,
  ) -> Arc<dyn DatafusionCatalogList> {
    Arc::new(DirectoryCatalogList {
      options: self.options.clone(),
      schemas,
      transaction,
    })
  }
}

pub struct DirectoryCatalogList {
  options: CatalogListOptions,
  schemas: Arc<Vec<String>>,
  transaction: TransactionHandle,
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
        schemas: self.schemas.clone(),
        transaction: self.transaction.clone(),
      }))
    } else {
      None
    }
  }
}
