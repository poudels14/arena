use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use arenasql::execution::{
  ExecutionPlanExtension, Privilege, SessionConfig, SessionContext,
  DEFAULT_SCHEMA_NAME,
};
use arenasql::runtime::RuntimeEnv;
use dashmap::DashMap;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use pgwire::api::ClientInfo;
use uuid::Uuid;

use super::storage::{ClusterStorageFactory, StorageOption};
use super::ClusterOptions;
use crate::auth::{
  AuthHeader, AuthenticatedSession, AuthenticatedSessionBuilder,
  AuthenticatedSessionStore,
};
use crate::error::{ArenaClusterError, ArenaClusterResult, Error};
use crate::extension::admin_exetension;
use crate::io::file::File;
use crate::pgwire::{ArenaPortalStore, ArenaQueryParser};
use crate::schema::{self, ADMIN_USERNAME, APPS_USERNAME, MANIFEST_FILE};
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
  pub(crate) jwt_secret: Option<String>,
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
      jwt_secret: options.jwt_secret.clone(),
    })
  }

  pub(crate) fn get_client_session<C: ClientInfo>(
    &self,
    client: &C,
  ) -> ArenaClusterResult<Arc<AuthenticatedSession>> {
    self
      .session_store
      .get_session(client.metadata().get("session_id").unwrap())
      .ok_or_else(|| ArenaClusterError::InvalidConnection)
  }

  pub(crate) fn get_or_create_new_session<C: ClientInfo>(
    &self,
    client: &C,
    header: &AuthHeader,
  ) -> ArenaClusterResult<Arc<AuthenticatedSession>> {
    // Only connection authenticated with apps user name can use
    // header auth
    if client
      .metadata()
      .get("user")
      .map(|u| u.as_str() != APPS_USERNAME)
      .unwrap_or(true)
    {
      return Err(Error::AuthenticationFailed);
    }
    match header {
      AuthHeader::Authenticated { session_id } => self
        .session_store
        .get_session(session_id)
        .ok_or_else(|| ArenaClusterError::InvalidConnection),
      AuthHeader::Token { token } => {
        if let Some(jwt_secret) = &self.jwt_secret {
          let verified_token = jsonwebtoken::decode::<serde_json::Value>(
            &token,
            &DecodingKey::from_secret((&jwt_secret).as_ref()),
            &Validation::new(Algorithm::HS512),
          );
          match verified_token {
            Ok(verified_token) => {
              let claims = verified_token
                .claims
                .as_object()
                .ok_or(Error::AuthenticationFailed)?;
              let user = claims.get("user").and_then(|u| u.as_str());
              let database = claims.get("database").and_then(|d| d.as_str());
              let session_id =
                claims.get("session_id").and_then(|s| s.as_str());

              match (user, database, session_id) {
                (Some(user), Some(db), Some(session_id)) => {
                  return self.create_new_session(
                    user.to_owned(),
                    Some(session_id.to_owned()),
                    db.to_owned(),
                    DEFAULT_SCHEMA_NAME.to_string(),
                    Privilege::TABLE_PRIVILEGES,
                  );
                }
                _ => {}
              }
            }
            _ => {}
          }
        }
        Err(Error::AuthenticationFailed)
      }
      _ => unreachable!(),
    }
  }

  pub(crate) fn create_new_session(
    &self,
    user: String,
    session_id: Option<String>,
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

    let (schemas, extensions): (Vec<String>, Vec<ExecutionPlanExtension>) =
      match user == ADMIN_USERNAME {
        true => (
          // Give access to "pg_catalog" for ADMIN users
          vec!["pg_catalog".to_owned(), schema],
          vec![Arc::new(admin_exetension)],
        ),
        false => (vec![schema], vec![]),
      };

    let session_context = SessionContext::new(
      SessionConfig {
        runtime: self.runtime.clone(),
        df_runtime: Default::default(),
        catalog: catalog.clone().into(),
        schemas: Arc::new(schemas),
        storage_factory,
        catalog_list_provider,
        execution_plan_extensions: Arc::new(extensions),
        privilege,
        ..Default::default()
      },
      Default::default(),
    );

    // Generate a random session_id if it's None
    // The auth request for app queries will have session_id set by the
    // client. Just the main postgres connections won't have it set
    let session_id = session_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let session = AuthenticatedSessionBuilder::default()
      .id(session_id)
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
