mod connection;
mod query;

use std::cell::RefCell;
use std::rc::Rc;

use anyhow::anyhow;
use anyhow::Result;
use deno_core::op2;
use deno_core::serde::Deserialize;
use deno_core::Extension;
use deno_core::Resource;
use deno_core::{Op, OpState, ResourceId};
use deno_unsync::JoinHandle;
use derivative::Derivative;
use tokio_postgres::Client;
use tracing::error;

use self::query::Param;
use self::query::QueryOptions;
use self::query::QueryResponse;
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
    port: i32,
    user: String,
    password: Option<String>,
    database: String,
    ssl: Option<bool>,
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
        user,
        password,
        database,
        ..
      } => match password {
        Some(password) => {
          format!(
            "postgresql://{}:{}@{}:{}/{}",
            user, password, host, port, database
          )
        }
        None => {
          format!("postgresql://{}@{}:{}/{}", user, host, port, database)
        }
      },
    }
  }

  pub fn ssl(&self) -> bool {
    match self {
      // TODO: parse string for ?ssl=true param
      ConnectionCredential::String(_) => false,
      ConnectionCredential::Config { ssl, .. } => ssl.unwrap_or(false),
    }
  }
}

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct PostgresConnectionResource {
  pub client: Rc<Client>,
  options: Option<QueryOptions>,
  #[derivative(Debug = "ignore")]
  pub handle: Rc<JoinHandle<()>>,
}

impl Resource for PostgresConnectionResource {
  // TODO(sagar): figure out how to close connection resource properly
  fn close(self: Rc<Self>) {
    self.handle.abort();
    drop(self);
  }
}

#[op2(async)]
#[smi]
pub async fn op_postgres_create_connection(
  state: Rc<RefCell<OpState>>,
  #[serde] config: ConnectionConfig,
) -> Result<ResourceId> {
  let cred_value = config
    .credential
    .ok_or_else(|| anyhow!("connectionString is missing from config"))?
    .get_value(&state.borrow())
    .map_err(|_| anyhow!("Failed to get database connection credential"))?;

  let credential =
    serde_json::from_value::<ConnectionCredential>(cred_value)
      .map_err(|_| anyhow!("unable to parse connection credential"))?;
  let connection_string = credential.to_connection_string();

  let (client, connection) =
    connection::create_connection(&connection_string, credential.ssl()).await?;

  let handle = deno_unsync::spawn(async move {
    if let Err(e) = connection.listen().await {
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

  query::execute_query(
    &resource.client,
    &query,
    &params,
    &options.unwrap_or_default(),
  )
  .await
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
