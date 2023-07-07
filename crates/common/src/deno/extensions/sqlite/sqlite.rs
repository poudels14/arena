use anyhow::{anyhow, Result};
use deno_core::serde::Deserialize;
use deno_core::serde_json::{Number, Value as JsonValue};
use rusqlite::types::{ToSqlOutput, Type, ValueRef};
use rusqlite::Error;
use rusqlite::ToSql;
use rusqlite::{types::Value, Row};
use rusqlite::{Connection, OpenFlags};
use std::path::Path;

#[derive(Default, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryOptions {
  /// Whether to update column names to camel case
  pub camel_case: bool,
}

pub fn create_connection(path: &str, flags: i32) -> Result<Connection> {
  Connection::open_with_flags(
    // TODO(sagar): check access to file
    Path::new(path),
    OpenFlags::from_bits(flags).unwrap_or(OpenFlags::SQLITE_OPEN_READ_ONLY),
  )
  .map_err(|e| anyhow!("{}", e))
}

#[derive(Clone, Debug, Deserialize)]
pub struct Param(JsonValue);

impl ToSql for Param {
  fn to_sql(&self) -> Result<ToSqlOutput<'_>, Error> {
    let value = match &self.0 {
      JsonValue::Null => Ok(Value::Null),
      JsonValue::Number(n) => {
        if let Some(v) = n.as_i64() {
          Ok(Value::Integer(v))
        } else if let Some(v) = n.as_f64() {
          Ok(Value::Real(v))
        } else {
          Err(Error::ToSqlConversionFailure(
            anyhow!("Error converting number to integer or float").into(),
          ))
        }
      }
      JsonValue::String(value) => Ok(Value::Text(value.to_owned())),
      _ => Err(Error::ToSqlConversionFailure(
        anyhow!("Unsupported data type: {}", self.0).into(),
      )),
    }?;
    Ok(ToSqlOutput::Owned(value))
  }
}

pub(crate) fn get_json_value(
  row: &Row,
  col_idx: usize,
) -> Result<JsonValue, Error> {
  let r = row.get_ref(col_idx);
  match r {
    Ok(value) => match value {
      ValueRef::Null => Ok(JsonValue::Null),
      ValueRef::Integer(i) => Ok(JsonValue::Number(i.into())),
      ValueRef::Real(f) => Number::from_f64(f)
        .ok_or(Error::FromSqlConversionFailure(
          col_idx,
          Type::Real,
          anyhow!("Failed to convert to JS number").into(),
        ))
        .map(|n| n.into()),
      ValueRef::Text(s) => Ok(JsonValue::String(unsafe {
        String::from_utf8_unchecked(s.to_vec())
      })),
      ValueRef::Blob(b) => Ok(JsonValue::String(base64::encode(b))),
    },
    Err(e) => Err(e),
  }
}
