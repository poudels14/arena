use std::fmt::Debug;
use std::sync::Arc;

use arenasql::arrow::as_string_array;
use arenasql::execution::{Privilege, SessionContext};
use arenasql::pgwire::api::auth::scram::{
  gen_salted_password, MakeSASLScramAuthStartupHandler,
  SASLScramAuthStartupHandler,
};
use arenasql::pgwire::api::auth::{
  AuthSource, DefaultServerParameterProvider, LoginInfo, Password,
  StartupHandler,
};
use arenasql::pgwire::api::{ClientInfo, MakeHandler};
use arenasql::pgwire::error::{PgWireError, PgWireResult};
use arenasql::pgwire::messages::{PgWireBackendMessage, PgWireFrontendMessage};
use async_trait::async_trait;
use derive_new::new;
use futures::Sink;
use log::debug;
use rand::Rng;

use crate::error::Error;
use crate::schema::{User, ADMIN_USERNAME, APPS_USERNAME, SYSTEM_CATALOG_NAME};
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

    let username = metadata
      .get("user")
      // idk if user is ever None :shrug:
      .ok_or_else(|| Error::UserDoesntExist("null".to_owned()))?;

    // Only check for the system users in cluster manifest if the connection
    // is trying to use system catalog
    if database == SYSTEM_CATALOG_NAME {
      self
        .cluster
        .manifest
        .get_user(&username)
        .ok_or_else(|| Error::UserDoesntExist(username.to_owned()))?;
    }

    // "apps" user shouldn't have any privilege by default
    // A proper privilege will be given to the queries if the
    // Auth header is verified for each query
    let privilege = if username == ADMIN_USERNAME {
      Privilege::SUPER_USER
    } else if database != SYSTEM_CATALOG_NAME {
      // If non-system catalog was authorized, provide table privileges
      Privilege::TABLE_PRIVILEGES
    } else {
      Privilege::NONE
    };

    let session = self.cluster.create_new_session(
      database,
      username.to_owned(),
      None,
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
    let username = login
      .user()
      .map_or_else(|| "admin".to_owned(), |d| d.clone());
    let catalog = login
      .database()
      .map(|s| s.to_owned())
      .unwrap_or(SYSTEM_CATALOG_NAME.to_owned());

    // If admin/apps username is used, authenticate them using cluster
    // manifest
    let password = if username == ADMIN_USERNAME || username == APPS_USERNAME {
      self
        .cluster
        .manifest
        .get_user(&username)
        .map(|user| user.password.to_owned())
    } else {
      // create a temporary session to query users from the catalog
      // that this connection is trying to access
      let session_context = self.cluster.create_session_context(
        &catalog,
        ADMIN_USERNAME,
        Privilege::SUPER_USER,
      )?;
      let userinfo =
        get_user_login_for_catalog(&session_context, &catalog, &username)
          .await
          .map_err(|_| {
            // Map all errors to auth error
            crate::error::Error::AuthenticationFailed
          })?;
      userinfo.map(|user| user.password.to_owned())
    };

    match password {
      Some(password) => {
        let salt = rand::thread_rng().gen::<[u8; 32]>().to_vec();
        let hash_password =
          gen_salted_password(&password, salt.as_ref(), ITERATIONS);
        Ok(Password::new(Some(salt), hash_password))
      }
      None => Err(Error::UserDoesntExist(username).into()),
    }
  }
}

async fn get_user_login_for_catalog(
  session_context: &SessionContext,
  catalog: &str,
  username: &str,
) -> arenasql::Result<Option<User>> {
  debug!("Checking user {:?} for catalog {:?}", username, catalog);
  let transaction = session_context.new_transaction()?;
  let rows = transaction
    .execute_sql(&format!(
      "EXECUTE arena_list_catalog_users('{}')",
      &urlencoding::encode(&catalog)
    ))
    .await?;

  let batches = rows.collect_batches().await?;
  Ok(
    batches
      .iter()
      .flat_map(|batch| {
        as_string_array(batch.column_by_name("catalog").unwrap())
          .iter()
          .zip(as_string_array(batch.column_by_name("user").unwrap()))
          .zip(as_string_array(batch.column_by_name("password").unwrap()))
          .filter_map(|((c, u), p)| c.zip(u).zip(p))
          .filter(|((c, u), _)| *c == catalog && *u == username)
          .map(|((_, user), password)| User {
            name: user.to_owned(),
            password: password.to_owned(),
            privilege: Privilege::TABLE_PRIVILEGES,
          })
      })
      .find(|u| u.name == username),
  )
}
