use anyhow::bail;
use anyhow::Result;
use deno_core::{
  op2, serde_json::Value, Extension, Op, OpState, Resource, ResourceId,
};
use heck::ToLowerCamelCase;
use rusqlite::params_from_iter;
use rusqlite::Connection;
use rusqlite::OpenFlags;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use tracing::debug;
mod sqlite;

use self::sqlite::get_json_value;
use self::sqlite::Param;
use self::sqlite::QueryOptions;
use super::r#macro::source_code;
use super::BuiltinExtension;
use crate::permissions;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension::new(
    Some(self::init()),
    vec![(
      "@arena/runtime/sqlite",
      source_code!(include_str!("./sqlite.js")),
    )],
  )
}

fn init() -> Extension {
  Extension {
    name: "arena/runtime/sqlite",
    ops: vec![
      op_sqlite_create_connection::DECL,
      op_sqlite_execute_query::DECL,
      op_sqlite_close_connection::DECL,
    ]
    .into(),
    enabled: true,
    ..Default::default()
  }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionConfig {
  path: String,
  flags: Option<i32>,
  options: Option<QueryOptions>,
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

#[derive(Debug)]
pub struct SqliteConnection {
  db_path: String,
  pub connection: RefCell<Option<Connection>>,
  options: QueryOptions,
}

impl Resource for SqliteConnection {
  fn close(self: Rc<Self>) {
    debug!("SqliteConnection dropped [db_path = {}]", self.db_path);
    drop(self);
  }
}

#[op2]
#[smi]
fn op_sqlite_create_connection(
  state: &mut OpState,
  #[serde] config: ConnectionConfig,
) -> Result<ResourceId> {
  let flags = config
    .flags
    .and_then(|f| OpenFlags::from_bits(f))
    .unwrap_or(OpenFlags::SQLITE_OPEN_READ_ONLY);

  let path = Path::new(&config.path);
  // Check access to db file
  if (flags
    & (OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE))
    .bits()
    != 0
  {
    permissions::resolve_write_path(state, path)?;
    path
      .parent()
      .map(|dir| {
        if !dir.exists() {
          std::fs::create_dir_all(dir)?;
        }
        Ok::<(), anyhow::Error>(())
      })
      .transpose()?;
  } else {
    permissions::resolve_read_path(state, path)?;
  }

  let connection = sqlite::create_connection(path, flags)?;
  let connection = SqliteConnection {
    db_path: path
      .canonicalize()
      .ok()
      .and_then(|p| p.to_str().map(|s| s.to_owned()))
      .expect("Failed to get db path"),
    connection: RefCell::new(Some(connection)),
    options: config.options.unwrap_or_default(),
  };

  debug!(
    "SqliteConnection created [db_path = {}]",
    &connection.db_path
  );
  Ok(state.resource_table.add::<SqliteConnection>(connection))
}

#[op2(async)]
#[serde]
async fn op_sqlite_execute_query(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
  #[string] query: String,
  #[serde] params: Vec<Param>,
  #[serde] options: Option<QueryOptions>,
) -> Result<QueryResponse> {
  let resource = state.borrow().resource_table.get::<SqliteConnection>(rid)?;
  let connection = resource.connection.borrow();
  if connection.is_none() {
    bail!("Connection is either not initialized or already closed");
  };

  let connection = connection.as_ref().unwrap();
  let mut stmt = connection.prepare_cached(&query)?;
  let cols_raw: Vec<String> = stmt
    .column_names()
    .iter()
    .map(|c| c.to_owned().to_owned())
    .collect::<Vec<String>>();

  let options = options.as_ref().unwrap_or(&resource.options);
  let cols = cols_raw
    .iter()
    .map(|c| {
      if options.camel_case {
        c.to_lower_camel_case()
      } else {
        c.to_string()
      }
    })
    .collect::<Vec<String>>();

  let mut rows = stmt.query(params_from_iter(params))?;
  let mut rows_vec: Vec<Vec<Value>> = Vec::new();
  let cols_len = cols.len();
  loop {
    match rows.next() {
      Ok(r) => {
        if let Some(row) = r {
          let mut r = Vec::with_capacity(cols_len);
          for idx in 0..cols_len {
            r.push(get_json_value(row, idx)?);
          }
          rows_vec.push(r)
        } else {
          break;
        }
      }
      Err(e) => bail!("{}", e),
    }
  }

  Ok(QueryResponse {
    columns: Columns {
      raw: cols_raw,
      values: cols,
    },
    rows: rows_vec,
  })
}

#[op2(async)]
async fn op_sqlite_close_connection(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
) -> Result<()> {
  let resource =
    { state.borrow().resource_table.get::<SqliteConnection>(rid)? };

  let connection = &mut resource.connection.borrow_mut().take();
  if connection.is_some() {
    let c = std::mem::take(connection);

    match c.unwrap().close() {
      Ok(_) => {
        let _ = state
          .borrow_mut()
          .resource_table
          .take::<SqliteConnection>(rid);
      }
      Err(e) => {
        resource.connection.borrow_mut().replace(e.0);
        bail!("{:?}", e.1);
      }
    }
  }
  Ok(())
}
