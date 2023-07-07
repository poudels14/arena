use self::sqlite::Param;
use self::sqlite::QueryOptions;
use super::BuiltinExtension;
use crate::deno::utils::fs;
use crate::resolve_from_file;
use anyhow::bail;
use anyhow::Result;
use deno_core::{
  op, serde_json::Value, Extension, OpState, Resource, ResourceId,
};
use heck::ToLowerCamelCase;
use rusqlite::params_from_iter;
use rusqlite::Connection;
use rusqlite::OpenFlags;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
pub mod sqlite;
use self::sqlite::get_json_value;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(self::init()),
    runtime_modules: vec![],
    snapshot_modules: vec![(
      "@arena/runtime/sqlite",
      resolve_from_file!("./sqlite.js"),
    )],
  }
}

fn init() -> Extension {
  Extension::builder("arena/runtime/sqlite")
    .ops(vec![
      op_sqlite_create_connection::decl(),
      op_sqlite_execute_query::decl(),
      op_sqlite_close_connection::decl(),
    ])
    .force_op_registration()
    .build()
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
  pub connection: RefCell<Option<Connection>>,
  options: QueryOptions,
}

impl Resource for SqliteConnection {
  fn close(self: Rc<Self>) {
    drop(self);
  }
}

#[op(fast)]
fn op_sqlite_create_connection(
  state: &mut OpState,
  config: ConnectionConfig,
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
    fs::resolve_write_path(state, path)?;
  } else {
    fs::resolve_read_path(state, path)?;
  }

  let connection = sqlite::create_connection(path, flags)?;
  let rid = state
    .resource_table
    .add::<SqliteConnection>(SqliteConnection {
      connection: RefCell::new(Some(connection)),
      options: config.options.unwrap_or_default(),
    });
  Ok(rid)
}

#[op(fast)]
async fn op_sqlite_execute_query(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  query: String,
  params: Vec<Param>,
  options: Option<QueryOptions>,
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

#[op(fast)]
async fn op_sqlite_close_connection(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
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
