use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use arenasql::execution::{
  AdvisoryLocks, ExecutionPlanExtension, Privilege, SessionConfig,
  SessionContext, SessionState, DEFAULT_SCHEMA_NAME,
};
use arenasql::pgwire::api::ClientInfo;
use arenasql::runtime::RuntimeEnv;
use arenasql::{Error as ArenaSqlError, Result as ArenaSqlResult};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use uuid::Uuid;

use super::storage::{ClusterStorageFactory, StorageOption};
use super::ClusterOptions;
use crate::auth::{
  AuthHeader, AuthenticatedSession, AuthenticatedSessionBuilder,
  AuthenticatedSessionStore,
};
use crate::error::{ArenaClusterError, ArenaClusterResult};
use crate::extension::admin_exetension;
use crate::io::file::File;
use crate::schema::{
  self, ADMIN_USERNAME, APPS_USERNAME, MANIFEST_FILE, SYSTEM_SCHEMA_NAME,
};
use crate::system::{
  ArenaClusterCatalogListProvider, CatalogListOptionsBuilder,
};

#[allow(unused)]
pub struct ArenaSqlCluster {
  pub(crate) manifest: Arc<schema::Cluster>,
  pub(crate) runtime: Arc<RuntimeEnv>,
  pub(crate) session_store: Arc<AuthenticatedSessionStore>,
  pub(crate) storage: Arc<ClusterStorageFactory>,
  pub(crate) jwt_secret: Option<String>,
  pub(crate) advisory_locks: Arc<AdvisoryLocks>,
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
      session_store: Arc::new(AuthenticatedSessionStore::new()),
      storage: Arc::new(ClusterStorageFactory::new(storage_options)),
      jwt_secret: options.jwt_secret.clone(),
      advisory_locks: Arc::new(AdvisoryLocks::new()),
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
      return Err(ArenaClusterError::AuthenticationFailed);
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
                .ok_or(ArenaClusterError::AuthenticationFailed)?;
              let user = claims.get("user").and_then(|u| u.as_str());
              let database = claims.get("database").and_then(|d| d.as_str());
              let session_id =
                claims.get("session_id").and_then(|s| s.as_str());

              match (user, database, session_id) {
                (Some(user), Some(db), Some(session_id)) => {
                  return self.create_new_session(
                    db.to_owned(),
                    user.to_owned(),
                    Some(session_id.to_owned()),
                    Privilege::TABLE_PRIVILEGES,
                  );
                }
                _ => {}
              }
            }
            _ => {}
          }
        }
        Err(ArenaClusterError::AuthenticationFailed)
      }
      _ => unreachable!(),
    }
  }

  pub(crate) fn create_new_session(
    &self,
    catalog: String,
    user: String,
    session_id: Option<String>,
    privilege: Privilege,
  ) -> ArenaClusterResult<Arc<AuthenticatedSession>> {
    let session_context =
      self.create_session_context(&catalog, &user, privilege)?;
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

  pub(crate) fn create_session_context(
    &self,
    catalog: &str,
    user: &str,
    privilege: Privilege,
  ) -> ArenaClusterResult<SessionContext> {
    Ok(Self::create_session_context_using_cluster_storage(
      self.storage.clone(),
      self.runtime.clone(),
      self.advisory_locks.clone(),
      catalog,
      user,
      privilege,
    )?)
  }

  #[tracing::instrument(
    skip(cluster_storage_factory, runtime),
    level = "TRACE"
  )]
  pub(crate) fn create_session_context_using_cluster_storage(
    cluster_storage_factory: Arc<ClusterStorageFactory>,
    runtime: Arc<RuntimeEnv>,
    advisory_locks: Arc<AdvisoryLocks>,
    catalog: &str,
    user: &str,
    privilege: Privilege,
  ) -> ArenaSqlResult<SessionContext> {
    let storage_factory = cluster_storage_factory
      .get_catalog(&catalog)?
      .ok_or_else(|| ArenaSqlError::DatabaseDoesntExist(catalog.to_owned()))?;

    let catalog_list_provider =
      Arc::new(ArenaClusterCatalogListProvider::with_options(
        CatalogListOptionsBuilder::default()
          .cluster_dir(cluster_storage_factory.options().root_dir().clone())
          .build()
          .unwrap(),
      ));

    let (schemas, extensions): (Vec<String>, Vec<ExecutionPlanExtension>) =
      match user == ADMIN_USERNAME {
        true => (
          // Give access to SYSTEM_SCHEMA_NAME for ADMIN users
          vec![
            SYSTEM_SCHEMA_NAME.to_owned(),
            DEFAULT_SCHEMA_NAME.to_owned(),
          ],
          vec![Arc::new(admin_exetension)],
        ),
        false => (vec![DEFAULT_SCHEMA_NAME.to_owned()], vec![]),
      };

    tracing::trace!("Using catalog [{:?}] schemas: {:?}", catalog, schemas);
    let mut session_state = SessionState::default();
    session_state.put(cluster_storage_factory.clone());
    session_state.put(runtime.clone());
    session_state.put(advisory_locks.clone());
    Ok(SessionContext::new(
      SessionConfig {
        runtime: runtime.clone(),
        df_runtime: Default::default(),
        catalog: catalog.into(),
        schemas: Arc::new(schemas),
        storage_factory,
        catalog_list_provider,
        execution_plan_extensions: Arc::new(extensions),
        privilege,
        advisory_locks,
        ..Default::default()
      },
      session_state,
    )?)
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
