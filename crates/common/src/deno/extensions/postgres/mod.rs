mod postgres;
mod tls;

pub use self::postgres::execute_query;
pub use self::postgres::Param;
use self::postgres::QueryOptions;
use super::BuiltinExtension;
use crate::deno::resources::env_variable::EnvironmentVariable;
use crate::resolve_from_file;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use deno_core::op;
use deno_core::serde::{Deserialize, Serialize};
use deno_core::Extension;
use deno_core::Resource;
use deno_core::{OpState, ResourceId};
use serde_json::Map;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::task::JoinHandle;
use tokio_postgres::Client;
use tracing::error;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(self::init()),
    runtime_modules: vec![],
    snapshot_modules: vec![(
      "@arena/runtime/postgres",
      resolve_from_file!("./postgres.js"),
    )],
  }
}

fn init() -> Extension {
  Extension::builder("arena/runtime/postgres")
    .ops(vec![
      op_postgres_create_connection::decl(),
      op_postgres_is_connected::decl(),
      op_postgres_execute_query::decl(),
    ])
    .build()
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionConfig {
  connection_string: Option<EnvironmentVariable>,

  options: Option<QueryOptions>,
}

#[derive(Clone, Debug, Serialize)]
pub struct QueryResponse {
  rows: Vec<Map<String, Value>>,
}

#[derive(Clone, Debug)]
pub struct PostgresConnectionResource {
  pub client: Rc<Client>,
  options: Option<QueryOptions>,
  pub handle: Rc<JoinHandle<()>>,
}

impl Resource for PostgresConnectionResource {
  // TODO(sagar): figure out how to close connection resource properly
  fn close(self: Rc<Self>) {
    self.handle.abort();
    drop(self);
  }
}

#[op]
pub async fn op_postgres_create_connection(
  state: Rc<RefCell<OpState>>,
  config: ConnectionConfig,
) -> Result<ResourceId> {
  let connection_string = match config.connection_string {
    Some(v) => v.get_value(&state.borrow()),
    None => bail!("connectionString is missing from config"),
  }?;

  let connection_string = connection_string
    .as_str()
    .ok_or(anyhow!("connectionString should be string"))?;

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
        options: config.options,
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
  options: Option<QueryOptions>,
) -> Result<QueryResponse> {
  let resource = state
    .borrow_mut()
    .resource_table
    .get::<PostgresConnectionResource>(rid)?;

  let defaut_options = QueryOptions::default();
  let rows = postgres::execute_query(
    &resource.client,
    &query,
    &params,
    options
      .as_ref()
      .unwrap_or(resource.options.as_ref().unwrap_or(&defaut_options)),
  )
  .await;

  Ok(QueryResponse { rows: rows? })
}
