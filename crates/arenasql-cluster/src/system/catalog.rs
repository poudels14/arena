use std::any::Any;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use arenasql::storage::Transaction;
use arenasql::{
  CatalogListProvider, DatafusionCatalogList, DatafusionCatalogProvider,
  SchemaProviderBuilder, SingleSchemaCatalogProvider,
};
use derive_builder::Builder;

use crate::schema::DEFAULT_SCHEMA_NAME;

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
    transaction: Transaction,
  ) -> Option<Arc<dyn DatafusionCatalogList>> {
    Some(Arc::new(DirectoryCatalogList {
      transaction,
      schema: Arc::new(DEFAULT_SCHEMA_NAME.to_string()),
      options: self.options.clone(),
    }))
  }
}

pub struct DirectoryCatalogList {
  schema: Arc<String>,
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

        let catalog = Arc::new(SingleSchemaCatalogProvider {
          schema: self.schema.clone(),
          schema_provider: Arc::new(
            SchemaProviderBuilder::default()
              .transaction(self.transaction.clone())
              .build()
              .unwrap(),
          ),
        });
        Some(catalog)
      }
    }
  }

  fn catalog_names(&self) -> Vec<String> {
    panic!("Listing catalog names not supported");
  }

  fn catalog(&self, name: &str) -> Option<Arc<dyn DatafusionCatalogProvider>> {
    if self.get_catalog_dir(&name).exists() {
      Some(Arc::new(SingleSchemaCatalogProvider {
        schema: self.schema.clone(),
        schema_provider: Arc::new(
          SchemaProviderBuilder::default()
            .transaction(self.transaction.clone())
            .build()
            .unwrap(),
        ),
      }))
    } else {
      None
    }
  }
}
