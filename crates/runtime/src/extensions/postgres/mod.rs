mod connection;
mod query;

use std::cell::RefCell;
use std::rc::Rc;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use deno_core::op2;
use deno_core::serde::{Deserialize, Serialize};
use deno_core::serde_json::Value;
use deno_core::Extension;
use deno_core::Resource;
use deno_core::{Op, OpState, ResourceId};
use heck::ToLowerCamelCase;
use tokio::task::JoinHandle;
use tokio_postgres::Client;
use tracing::error;

use self::query::Param;
use super::r#macro::include_source_code;
use super::BuiltinExtension;
use crate::env::EnvironmentVariable;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension::new(
    Some(self::init()),
    vec![(
      "@arena/runtime/postgres",
      include_source_code!("./postgres.js"),
    )],
  )
}

fn init() -> Extension {
  Extension {
    name: "arena/runtime/postgres",
    ops: vec![
      op_postgres_create_connection::DECL,
      op_postgres_is_connected::DECL,
      op_postgres_execute_query::DECL,
    ]
    .into(),
    enabled: true,
    ..Default::default()
  }
}

#[derive(Default, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryOptions {
  /// Whether to update column names to camel case
  pub camel_case: bool,
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

  /// Only set if the query returns rows
  row_count: Option<u64>,

  modified_rows: Option<u64>,
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

#[op2(async)]
#[smi]
pub async fn op_postgres_create_connection(
  state: Rc<RefCell<OpState>>,
  #[serde] config: ConnectionConfig,
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
    connection::create_connection(&connection_string).await?;

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

#[op2(fast)]
pub fn op_postgres_is_connected(
  state: &mut OpState,
  #[smi] rid: ResourceId,
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

#[op2(async)]
#[serde]
pub async fn op_postgres_execute_query(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
  #[string] query: String,
  #[serde] params: Option<Vec<Param>>,
  #[serde] options: Option<QueryOptions>,
) -> Result<QueryResponse> {
  let resource = state
    .borrow_mut()
    .resource_table
    .get::<PostgresConnectionResource>(rid)?;

  let query_response =
    query::execute_query(&resource.client, &query, &params).await?;

  let defaut_options = QueryOptions::default();
  let camel_case = options
    .as_ref()
    .unwrap_or(resource.options.as_ref().unwrap_or(&defaut_options))
    .camel_case;

  let cased_column = query_response
    .columns
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
      raw: query_response.columns,
      values: cased_column,
    },
    rows: query_response.rows,
    modified_rows: query_response.modified_rows,
    row_count: query_response.row_count,
  })
}

#[op2(fast)]
pub fn op_postgres_close(
  state: &mut OpState,
  #[smi] rid: ResourceId,
) -> Result<()> {
  let _ = state
    .resource_table
    .take::<PostgresConnectionResource>(rid)
    .ok();
  Ok(())
}
