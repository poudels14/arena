mod execution;
mod storage;

use std::path::PathBuf;
use std::sync::Arc;

use arenasql::runtime::RuntimeEnv;
use arenasql::storage::{Serializer, StorageFactoryBuilder};
use arenasql::{SessionConfig, SessionContext, SingleCatalogListProvider};
use tokio::sync::Mutex;
use uuid::Uuid;

use self::storage::StorageFactory;
use crate::auth::{AuthenticatedSession, AuthenticatedSessionStore};
use crate::error::{ArenaClusterError, ArenaClusterResult};
use crate::pgwire::{ArenaPortalStore, ArenaQueryParser, QueryClient};

pub struct ArenaSqlCluster {
  pub(crate) runtime: Arc<RuntimeEnv>,
  pub(crate) parser: Arc<ArenaQueryParser>,
  pub(crate) poral_store: Arc<ArenaPortalStore>,
  pub(crate) session_store: Arc<AuthenticatedSessionStore>,
  pub(crate) storage: Arc<StorageFactory>,
}

impl ArenaSqlCluster {
  pub fn new(path: &str) -> Self {
    Self {
      runtime: Arc::new(RuntimeEnv::default()),
      parser: Arc::new(ArenaQueryParser {}),
      poral_store: Arc::new(ArenaPortalStore::new()),
      session_store: Arc::new(AuthenticatedSessionStore::new()),
      storage: Arc::new(StorageFactory::new(PathBuf::from(path))),
    }
  }

  fn get_or_create_new_session(
    &self,
    client: &QueryClient,
  ) -> ArenaClusterResult<Arc<AuthenticatedSession>> {
    match client {
      QueryClient::Authenticated { id } => self
        .session_store
        .get(id)
        .ok_or_else(|| ArenaClusterError::InvalidConnection),
      QueryClient::New { user, database } => self.create_new_session(
        user.clone(),
        database.clone(),
        "public".to_owned(),
      ),
    }
  }

  pub(crate) fn create_new_session(
    &self,
    user: String,
    catalog: String,
    schema: String,
  ) -> ArenaClusterResult<Arc<AuthenticatedSession>> {
    let storage_provider = self
      .storage
      .get(&catalog)?
      .ok_or_else(|| ArenaClusterError::CatalogNotFound(catalog.clone()))?;

    let catalog_list_provider =
      Arc::new(SingleCatalogListProvider::new(&catalog, &schema));

    let session_context = SessionContext::with_config(SessionConfig {
      runtime: self.runtime.clone(),
      df_runtime: Default::default(),
      catalog: catalog.to_string(),
      default_schema: schema.clone(),
      storage_factory: StorageFactoryBuilder::default()
        .catalog(catalog.clone())
        .serializer(Serializer::VarInt)
        .kv_provider(storage_provider)
        .build()
        .unwrap()
        .into(),
      catalog_list_provider,
      ..Default::default()
    });

    let session_id = Uuid::new_v4().to_string();
    let session = AuthenticatedSession {
      id: session_id.clone(),
      database: catalog,
      user: user.to_string(),
      ctxt: session_context,
      transaction: Arc::new(Mutex::new(None)),
    };
    Ok(self.session_store.put(session))
  }
}
