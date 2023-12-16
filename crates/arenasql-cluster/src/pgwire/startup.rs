use std::fmt::Debug;

use async_trait::async_trait;
use futures::Sink;
use pgwire::api::auth::{
  finish_authentication, save_startup_parameters_to_metadata,
  DefaultServerParameterProvider, StartupHandler,
};
use pgwire::api::ClientInfo;
use pgwire::error::PgWireResult;
use pgwire::messages::{PgWireBackendMessage, PgWireFrontendMessage};

use crate::server::cluster::ArenaSqlCluster;

#[async_trait]
impl StartupHandler for ArenaSqlCluster {
  async fn on_startup<C>(
    &self,
    client: &mut C,
    message: PgWireFrontendMessage,
  ) -> PgWireResult<()>
  where
    C: ClientInfo + Send + Unpin + Sink<PgWireBackendMessage>,
    C::Error: Debug,
  {
    if let PgWireFrontendMessage::Startup(ref startup) = message {
      save_startup_parameters_to_metadata(client, startup);
      let metadata = client.metadata_mut();
      let database = metadata
        .get("database")
        .map_or_else(|| "postgres".to_owned(), |d| d.clone());
      let user = metadata
        .get("user")
        .map_or_else(|| "root".to_owned(), |d| d.clone());

      // TODO: authenticate

      let session =
        self.create_new_session(user, database, "public".to_owned())?;
      metadata.insert("session_id".to_owned(), session.id.to_string());

      finish_authentication(client, &DefaultServerParameterProvider::default())
        .await;
    }
    Ok(())
  }
}
