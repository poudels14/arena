mod postgres;
mod tls;

pub use self::postgres::execute_query;
pub use self::postgres::Param;
use crate::deno::SecretResource;
use anyhow::anyhow;
use anyhow::Result;
use deno_core::op;
use deno_core::serde::{Deserialize, Serialize};
use deno_core::Extension;
use deno_core::ExtensionFileSource;
use deno_core::ExtensionFileSourceCode;
use deno_core::Resource;
use deno_core::{OpState, ResourceId};
use serde_json::Map;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::task::JoinHandle;
use tokio_postgres::Client;
use tracing::error;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionOptions {
  /**
   * The resource id of the connection string secret
   */
  connection_string_id: Option<ResourceId>,

  /**
   * Raw connection string
   */
  connection_string: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct QueryResponse {
  rows: Vec<Map<String, Value>>,
}

#[derive(Clone, Debug)]
pub struct PostgresConnectionResource {
  pub client: Rc<Client>,
  pub handle: Rc<JoinHandle<()>>,
}

impl Resource for PostgresConnectionResource {
  // TODO(sagar): figure out how to close connection resource properly
  fn close(self: Rc<Self>) {
    self.handle.abort();
    drop(self);
  }
}

pub fn init() -> Extension {
  Extension::builder("postgres")
    .ops(vec![
      op_postgres_create_connection::decl(),
      op_postgres_is_connected::decl(),
      op_postgres_execute_query::decl(),
    ])
    .build()
}

pub fn get_modules_for_snapshotting() -> Vec<ExtensionFileSource> {
  vec![ExtensionFileSource {
    specifier: "@arena/postgres".to_owned(),
    code: ExtensionFileSourceCode::LoadedFromFsDuringSnapshot(
      format!(
        "{}/src/deno/extensions/postgres/postgres.js",
        env!("CARGO_MANIFEST_DIR")
      )
      .into(),
    ),
  }]
}

#[op]
pub async fn op_postgres_create_connection(
  state: Rc<RefCell<OpState>>,
  options: ConnectionOptions,
) -> Result<ResourceId> {
  let connection_string = match options.connection_string_id {
    Some(rid) => {
      let secret = state
        .borrow_mut()
        .resource_table
        .get::<SecretResource>(rid)?;
      secret
        .value
        .as_str()
        .map(|s| s.to_string())
        .ok_or(anyhow!("Failed to get connection string from secret store"))
    }
    None => {
      // fallback to connection string
      options
        .connection_string
        .ok_or(anyhow!("Connection credentials not set"))
    }
  }?;

  let (client, connection) =
    postgres::create_connection(&connection_string).await?;

  let handle = tokio::spawn(async {
    if let Err(e) = connection.await {
      error!("connection error: {}", e);
    }
  });

  let resource_id =
    state
      .borrow_mut()
      .resource_table
      .add(PostgresConnectionResource {
        client: Rc::new(client),
        handle: Rc::new(handle),
      });

  Ok(resource_id)
}

#[op]
pub fn op_postgres_is_connected(
  state: &mut OpState,
  rid: ResourceId,
) -> Result<bool> {
  let resource = state
    .resource_table
    .get::<PostgresConnectionResource>(rid)
    .ok();
  Ok(
    !resource
      .and_then(|r| Some(r.client.is_closed()))
      .unwrap_or(true),
  )
}

#[op]
pub async fn op_postgres_execute_query(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  query: String,
  params: Vec<postgres::Param>,
) -> Result<QueryResponse> {
  let resource = state
    .borrow_mut()
    .resource_table
    .get::<PostgresConnectionResource>(rid)?;

  let rows = postgres::execute_query(&resource.client, &query, &params).await;

  Ok(QueryResponse { rows: rows? })
}
