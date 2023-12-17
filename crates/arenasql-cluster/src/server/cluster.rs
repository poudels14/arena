use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use arenasql::runtime::RuntimeEnv;
use arenasql::{SessionConfig, SessionContext, DEFAULT_SCHEMA_NAME};
use dashmap::DashMap;
use derivative::Derivative;
use pgwire::api::ClientInfo;

use super::storage::{StorageFactory, StorageOption};
use crate::auth::{
  AuthenticatedSession, AuthenticatedSessionBuilder, AuthenticatedSessionStore,
};
use crate::error::{ArenaClusterError, ArenaClusterResult};
use crate::io::file::File;
use crate::pgwire::{ArenaPortalStore, ArenaQueryParser, QueryClient};
use crate::schema::{self, MANIFEST_FILE};
use crate::system::{
  ArenaClusterCatalogListProvider, CatalogListOptionsBuilder,
};

#[allow(unused)]
pub struct ArenaSqlCluster {
  pub(crate) manifest: Arc<schema::Cluster>,
  pub options: ClusterOptions,
  pub(crate) runtime: Arc<RuntimeEnv>,
  pub(crate) parser: Arc<ArenaQueryParser>,
  /// Portal stores should be unique to each session since different
  /// statements can be stored under same default name and sharing
  /// portals across sessions would lead to stored statements being
  /// overridden
  pub(crate) poral_stores: Arc<DashMap<String, Arc<ArenaPortalStore>>>,
  pub(crate) session_store: Arc<AuthenticatedSessionStore>,
  pub(crate) storage: Arc<StorageFactory>,
}

#[derive(Debug, Derivative)]
pub struct ClusterOptions {
  /// Location of database data directory
  pub dir: Arc<PathBuf>,

  /// Per database cache size in MB
  #[derivative(Default(value = "10"))]
  pub cache_size_mb: usize,
}

impl ArenaSqlCluster {
  pub fn load(config: ClusterOptions) -> Result<Self> {
    let manifest = File::read(&config.dir.join(MANIFEST_FILE))
      .context("Error reading cluster manifest")?;
    Ok(Self {
      manifest,
      runtime: Arc::new(RuntimeEnv::default()),
      parser: Arc::new(ArenaQueryParser {}),
      poral_stores: Arc::new(DashMap::new()),
      session_store: Arc::new(AuthenticatedSessionStore::new()),
      storage: Arc::new(StorageFactory::new(config.dir.to_path_buf())),
      options: config,
    })
  }

  pub(crate) fn get_client_session<C: ClientInfo>(
    &self,
    client: &C,
  ) -> ArenaClusterResult<Arc<AuthenticatedSession>> {
    self
      .session_store
      .get_session(
        client
          .metadata()
          .get("session_id")
          .unwrap()
          .parse::<u64>()
          .unwrap(),
      )
      .ok_or_else(|| ArenaClusterError::InvalidConnection)
  }

  pub(crate) fn get_or_create_new_session(
    &self,
    client: &QueryClient,
  ) -> ArenaClusterResult<Arc<AuthenticatedSession>> {
    match client {
      QueryClient::Authenticated { id } => self
        .session_store
        .get_session(*id)
        .ok_or_else(|| ArenaClusterError::InvalidConnection),
      QueryClient::New { user, database } => self.create_new_session(
        user.clone(),
        database.clone(),
        DEFAULT_SCHEMA_NAME.to_string(),
      ),
      _ => unreachable!(),
    }
  }

  pub(crate) fn create_new_session(
    &self,
    user: String,
    catalog: String,
    schema: String,
  ) -> ArenaClusterResult<Arc<AuthenticatedSession>> {
    let storage_factory = self
      .storage
      .get_catalog(
        &catalog,
        StorageOption {
          cache_size_mb: Some(self.options.cache_size_mb),
        },
      )?
      .ok_or_else(|| ArenaClusterError::CatalogNotFound(catalog.clone()))?;

    let catalog_list_provider =
      Arc::new(ArenaClusterCatalogListProvider::with_options(
        CatalogListOptionsBuilder::default()
          .cluster_dir(self.options.dir.clone())
          .build()
          .unwrap(),
      ));

    let session_context = SessionContext::with_config(SessionConfig {
      runtime: self.runtime.clone(),
      df_runtime: Default::default(),
      catalog: catalog.clone().into(),
      schemas: Arc::new(vec![schema]),
      storage_factory,
      catalog_list_provider,
      ..Default::default()
    });

    let session = AuthenticatedSessionBuilder::default()
      .id(self.session_store.generate_session_id())
      .database(catalog)
      .user(user.to_string())
      .context(session_context)
      .build()
      .unwrap();
    Ok(self.session_store.put(session))
  }
}
