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
use deno_core::serde_json::Value;
use deno_core::Extension;
use deno_core::Resource;
use deno_core::{OpState, ResourceId};
use heck::ToLowerCamelCase;
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
    .force_op_registration()
    .build()
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionConfig {
  credential: Option<EnvironmentVariable>,
  options: Option<QueryOptions>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
enum ConnectionCredential {
  String(String),
  Config {
    host: String,
    port: String,
    username: String,
    password: String,
    database: String,
  },
}

impl ConnectionCredential {
  pub fn to_connection_string(&self) -> String {
    match self {
      ConnectionCredential::String(connection_string) => {
        connection_string.to_owned()
      }
      ConnectionCredential::Config {
        host,
        port,
        username,
        password,
        database,
      } => format!(
        "postgresql://{}:{}@{}:{}/{}",
        username, password, host, port, database
      ),
    }
  }
}

#[derive(Clone, Debug, Serialize)]
pub struct QueryResponse {
  columns: Columns,
  /**
   * Note(sagar): send data as array since sending as Object is almost
   * 4x slower than sending as array and reducing the array as objects
   * on JS side. Repeating column names for each row/col also probably
   * added to the serialization cost
   */
  rows: Vec<Vec<Value>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Columns {
  /**
   * Raw name of the columns
   */
  raw: Vec<String>,
  /**
   * Formatted column names
   * For example, these are CamedCased if `queryoptions.camel_case` is true
   */
  values: Vec<String>,
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
  let connection_string = match config.credential {
    Some(cred) => {
      let cred_value = cred
        .get_value(&state.borrow())
        .map_err(|_| anyhow!("Failed to get database connection credential"))?;

      serde_json::from_value::<ConnectionCredential>(cred_value)
        .map_err(|_| anyhow!("unable to parse connection credential"))?
        .to_connection_string()
    }
    None => bail!("connectionString is missing from config"),
  };

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

  let (cols_raw, rows) =
    postgres::execute_query(&resource.client, &query, &params).await?;

  let defaut_options = QueryOptions::default();
  let camel_case = options
    .as_ref()
    .unwrap_or(resource.options.as_ref().unwrap_or(&defaut_options))
    .camel_case;

  let cols = cols_raw
    .iter()
    .map(|c| {
      if camel_case {
        c.to_lower_camel_case()
      } else {
        c.to_string()
      }
    })
    .collect();

  Ok(QueryResponse {
    columns: Columns {
      raw: cols_raw,
      values: cols,
    },
    rows,
  })
}
