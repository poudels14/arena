use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use arenasql::execution::{
  Privilege, SessionConfig, SessionContext, DEFAULT_SCHEMA_NAME,
};
use arenasql::runtime::RuntimeEnv;
use dashmap::DashMap;
use pgwire::api::ClientInfo;

use super::storage::{ClusterStorageFactory, StorageOption};
use super::ClusterOptions;
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
  pub(crate) runtime: Arc<RuntimeEnv>,
  pub(crate) parser: Arc<ArenaQueryParser>,
  /// Portal stores should be unique to each session since different
  /// statements can be stored under same default name and sharing
  /// portals across sessions would lead to stored statements being
  /// overridden
  pub(crate) poral_stores: Arc<DashMap<String, Arc<ArenaPortalStore>>>,
  pub(crate) session_store: Arc<AuthenticatedSessionStore>,
  pub(crate) storage: Arc<ClusterStorageFactory>,
}

impl ArenaSqlCluster {
  pub fn load(options: &ClusterOptions) -> Result<Self> {
    let root_dir = Path::new(&options.root).to_path_buf();

    let manifest = File::read(&root_dir.join(MANIFEST_FILE))
      .context("Error reading cluster manifest")?;

    let backup_dir = options
      .backup_dir
      .as_ref()
      .map(|p| create_path_if_not_exists(&p))
      .transpose()?;
    let checkpoint_dir = options
      .checkpoint_dir
      .as_ref()
      .map(|p| create_path_if_not_exists(&p))
      .transpose()?;

    let mut storage_options = StorageOption::default();
    storage_options
      .set_backup_dir(backup_dir)
      .set_checkpoint_dir(checkpoint_dir)
      .set_cache_size_mb(Some(options.cache_size_mb))
      .set_root_dir(root_dir.into());

    Ok(Self {
      manifest,
      runtime: Arc::new(RuntimeEnv::default()),
      parser: Arc::new(ArenaQueryParser {}),
      poral_stores: Arc::new(DashMap::new()),
      session_store: Arc::new(AuthenticatedSessionStore::new()),
      storage: Arc::new(ClusterStorageFactory::new(storage_options)),
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
      QueryClient::Authenticated { session_id: id } => self
        .session_store
        .get_session(*id)
        .ok_or_else(|| ArenaClusterError::InvalidConnection),
      QueryClient::New { user, database } => self.create_new_session(
        user.clone(),
        database.clone(),
        DEFAULT_SCHEMA_NAME.to_string(),
        Privilege::TABLE_PRIVILEGES,
      ),
      _ => unreachable!(),
    }
  }

  pub(crate) fn create_new_session(
    &self,
    user: String,
    catalog: String,
    schema: String,
    privilege: Privilege,
  ) -> ArenaClusterResult<Arc<AuthenticatedSession>> {
    let storage_factory = self
      .storage
      .get_catalog(&catalog)?
      .ok_or_else(|| ArenaClusterError::CatalogNotFound(catalog.clone()))?;

    let catalog_list_provider =
      Arc::new(ArenaClusterCatalogListProvider::with_options(
        CatalogListOptionsBuilder::default()
          .cluster_dir(self.storage.options().root_dir().clone())
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
      privilege,
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

  pub async fn graceful_shutdown(&self) -> Result<()> {
    // Need to remove all sessions from the store first so that
    // all active transactions are dropped
    self.session_store.clear();
    Ok(self.storage.graceful_shutdown().await?)
  }
}

fn create_path_if_not_exists(path: &str) -> Result<PathBuf> {
  let p = PathBuf::from(path);
  if !p.exists() {
    std::fs::create_dir_all(&p)
      .context(format!("Failed to create dir: {:?}", p))?;
  }
  p.canonicalize()
    .context(format!("Failed to canonicalize path: {:?}", p))
}
