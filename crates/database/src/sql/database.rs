use anyhow::{anyhow, Result};
use libsql::Params;
use libsql_sys::ValueType;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

pub struct SqliteDatabase {
  pub(super) connection: libsql::Connection,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Column {
  name: String,
  #[serde(rename = "type")]
  typ: String,
}

pub fn open(path: &str) -> Result<SqliteDatabase> {
  let db = libsql::Database::open(path)
    .map_err(|_| anyhow!("Failed to open database"))?;

  let connection = db.connect().map_err(|e| anyhow!("{}", e))?;

  Ok(SqliteDatabase { connection })
}

impl SqliteDatabase {
  pub async fn query(
    &self,
    stmt: &str,
    params: Params,
  ) -> Result<(Vec<Column>, Vec<Vec<Value>>)> {
    let mut result = self.connection.query(&stmt, params).await?;

    let column_count = result.column_count();
    let mut columns = Vec::with_capacity(column_count as usize);

    for idx in 0..column_count {
      columns.push(Column {
        name: result.column_name(idx).unwrap().to_string(),
        typ: to_column_type_str(result.column_type(idx)?),
      });
    }

    let mut rows = vec![];
    while let Some(r) = result.next()? {
      let mut row = Vec::with_capacity(column_count as usize);
      for idx in 0..column_count {
        row.push(to_serde_json(r.get_value(idx)?)?);
      }
      rows.push(row);
    }

    Ok((columns, rows))
  }
}

fn to_column_type_str(t: ValueType) -> String {
  match t {
    ValueType::Integer => "INTEGER",
    ValueType::Real => "REAL",
    ValueType::Text => "TEXT",
    ValueType::Blob => "BLOB",
    ValueType::Null => "NULL",
  }
  .to_owned()
}

fn to_serde_json(value: libsql::Value) -> Result<Value> {
  match value {
    libsql::Value::Integer(v) => Ok(Value::Number(v.into())),
    libsql::Value::Real(v) => Number::from_f64(v)
      .map(|v| Value::Number(v))
      .ok_or(anyhow!("Error converting float")),
    libsql::Value::Text(v) => Ok(Value::String(v)),
    libsql::Value::Blob(v) => Ok(Value::String(base64::encode(v))),
    libsql::Value::Null => Ok(Value::Null),
  }
}
