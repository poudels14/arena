use std::fmt::Debug;
use std::sync::Arc;

use arenasql::execution::DEFAULT_SCHEMA_NAME;
use async_trait::async_trait;
use derive_new::new;
use futures::Sink;
use pgwire::api::auth::scram::{
  gen_salted_password, MakeSASLScramAuthStartupHandler,
  SASLScramAuthStartupHandler,
};
use pgwire::api::auth::{
  AuthSource, DefaultServerParameterProvider, LoginInfo, Password,
  StartupHandler,
};
use pgwire::api::{ClientInfo, MakeHandler};
use pgwire::error::{PgWireError, PgWireResult};
use pgwire::messages::{PgWireBackendMessage, PgWireFrontendMessage};
use rand::Rng;

use crate::error::Error;
use crate::schema::SYSTEM_CATALOG_NAME;
use crate::server::ArenaSqlCluster;

pub const ITERATIONS: usize = 64_000;

pub struct ArenaSqlClusterAuthenticator {
  cluster: Arc<ArenaSqlCluster>,
  authenticator: MakeSASLScramAuthStartupHandler<
    ArenaAuthSource,
    DefaultServerParameterProvider,
  >,
}

impl ArenaSqlClusterAuthenticator {
  pub fn new(cluster: Arc<ArenaSqlCluster>) -> Self {
    let mut authenticator = MakeSASLScramAuthStartupHandler::new(
      Arc::new(ArenaAuthSource::new(cluster.clone())),
      Arc::new(DefaultServerParameterProvider::default()),
    );
    authenticator.set_iterations(ITERATIONS);

    Self {
      authenticator,
      cluster,
    }
  }
}

impl MakeHandler for ArenaSqlClusterAuthenticator {
  type Handler = Arc<ArenaAuthHandler>;

  fn make(&self) -> Self::Handler {
    Arc::new(ArenaAuthHandler::new(
      self.cluster.clone(),
      self.authenticator.make(),
    ))
  }
}

#[derive(new, Clone)]
pub struct ArenaAuthHandler {
  cluster: Arc<ArenaSqlCluster>,
  scram: Arc<
    SASLScramAuthStartupHandler<
      ArenaAuthSource,
      DefaultServerParameterProvider,
    >,
  >,
}

#[async_trait]
impl StartupHandler for ArenaAuthHandler {
  async fn on_startup<C>(
    &self,
    client: &mut C,
    message: PgWireFrontendMessage,
  ) -> PgWireResult<()>
  where
    C: ClientInfo + Send + Unpin + Sink<PgWireBackendMessage>,
    C::Error: Debug,
    PgWireError: From<<C as Sink<PgWireBackendMessage>>::Error>,
  {
    self.scram.on_startup(client, message).await?;

    let metadata = client.metadata_mut();
    let database = metadata
      .get("database")
      .map_or_else(|| SYSTEM_CATALOG_NAME.to_owned(), |d| d.clone());
    let user_name = metadata
      .get("user")
      .map_or_else(|| "admin".to_owned(), |d| d.clone());
    let session = self.cluster.create_new_session(
      user_name,
      database,
      DEFAULT_SCHEMA_NAME.to_owned(),
    )?;
    metadata.insert("session_id".to_owned(), session.id().to_string());
    Ok(())
  }
}

#[derive(new, Clone)]
pub struct ArenaAuthSource {
  cluster: Arc<ArenaSqlCluster>,
}

#[async_trait]
impl AuthSource for ArenaAuthSource {
  async fn get_password(&self, login: &LoginInfo) -> PgWireResult<Password> {
    let database = login
      .database()
      .map_or_else(|| SYSTEM_CATALOG_NAME.to_owned(), |d| d.clone());
    let user_name = login
      .user()
      .map_or_else(|| "admin".to_owned(), |d| d.clone());

    let password = if database == SYSTEM_CATALOG_NAME {
      self
        .cluster
        .manifest
        .get_user(&user_name)
        .map(|m| m.password.clone())
    } else {
      None
    };

    // If matching password isn't found, it means the user
    // wasn't found
    match password {
      Some(password) => {
        let salt = rand::thread_rng().gen::<[u8; 32]>().to_vec();
        let hash_password =
          gen_salted_password(&password, salt.as_ref(), ITERATIONS);
        Ok(Password::new(Some(salt), hash_password))
      }
      None => Err(Error::UserDoesntExist(user_name).into()),
    }
  }
}
