use std::fmt::Debug;
use std::sync::Arc;

use arenasql::execution::{Privilege, DEFAULT_SCHEMA_NAME};
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
use crate::schema::{ADMIN_USERNAME, SYSTEM_CATALOG_NAME};
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
    let user = metadata
      .get("user")
      .and_then(|name| self.cluster.manifest.get_user(&name))
      // idk if user is ever None :shrug:
      .ok_or_else(|| Error::UserDoesntExist("null".to_owned()))?;

    // "apps" user shouldn't have any privilege by default
    // A proper privilege will be given to the queries if the
    // Auth header is verified for each query
    let privilege = if user.name == ADMIN_USERNAME {
      Privilege::SUPER_USER
    } else {
      Privilege::NONE
    };

    let session = self.cluster.create_new_session(
      user.name.to_owned(),
      None,
      database,
      DEFAULT_SCHEMA_NAME.to_owned(),
      privilege,
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
    // Note: no need to check user and database combination since all postgres
    // auth is done against "system" users who will have access to all databases
    // and another level auth will be done for Arena session
    let user_name = login
      .user()
      .map_or_else(|| "admin".to_owned(), |d| d.clone());

    let user = self.cluster.manifest.get_user(&user_name);
    match user {
      Some(ref user) => {
        let salt = rand::thread_rng().gen::<[u8; 32]>().to_vec();
        let hash_password =
          gen_salted_password(&user.password, salt.as_ref(), ITERATIONS);
        Ok(Password::new(Some(salt), hash_password))
      }
      None => Err(Error::UserDoesntExist(user_name).into()),
    }
  }
}
