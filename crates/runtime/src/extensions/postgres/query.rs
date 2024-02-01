use std::fmt::Write;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Error;
use anyhow::Result;
use bytes::BufMut;
use bytes::BytesMut;
use chrono::NaiveDateTime;
use deno_core::serde_json::{json, Value};
use futures::TryStreamExt;
use heck::ToLowerCamelCase;
use postgres::types::ToSql;
use postgres::types::Type;
use serde::{Deserialize, Serialize};
use tokio_postgres::types::IsNull;
use tokio_postgres::Client;
use tokio_postgres::{Column, Row};
use tracing::error;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize)]
pub struct Param(Value);

#[derive(Default, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryOptions {
  /// Whether to update column names to camel case
  pub camel_case: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponse {
  /// Only set if the query returns rows
  row_count: Option<u64>,

  /**
   * Note(sagar): send data as array since sending as Object is almost
   * 4x slower than sending as array and reducing the array as objects
   * on JS side. Repeating column names for each row/col also probably
   * added to the serialization cost
   */
  rows: Vec<Vec<Value>>,

  fields: Vec<Field>,

  #[serde(skip_serializing_if = "Option::is_none")]
  modified_rows: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
  name: String,

  /// Camel cased name if applicable
  #[serde(
    rename(serialize = "_casedName"),
    skip_serializing_if = "Option::is_none"
  )]
  _cased_name: Option<String>,

  #[serde(rename(serialize = "dataTypeID"))]
  data_type_id: u32,
}

#[tracing::instrument(skip(client, params), level = "trace")]
pub async fn execute_query(
  client: &Client,
  query: &str,
  params: Option<Vec<Param>>,
  options: &QueryOptions,
) -> Result<QueryResponse, Error> {
  let mut response = QueryResponse {
    row_count: None,
    rows: vec![],
    fields: vec![],
    modified_rows: None,
  };
  let res: Vec<Row> =
    match client.query_raw(query, &params.unwrap_or_default()).await {
      // TODO: stream the response?
      Ok(stream) => match stream.try_collect().await {
        Ok(data) => Ok::<Vec<Row>, anyhow::Error>(data),
        Err(err) => bail!("Error reading rows stream: {}", err.to_string()),
      },
      Err(err) => {
        bail!("Error executing query: {}", err.to_string())
      }
    }?;

  response.fields = res
    .get(0)
    .map(|row| {
      row
        .columns()
        .iter()
        .map(|c| Field {
          name: c.name().to_string(),
          _cased_name: options
            .camel_case
            .filter(|c| *c)
            .map(|_| c.name().to_lower_camel_case()),
          data_type_id: c.type_().oid(),
        })
        .collect()
    })
    .unwrap_or_default();

  response.rows = res
    .iter()
    .map(|row: &Row| {
      row
        .columns()
        .iter()
        .enumerate()
        .map(|(index, column)| get_json_value(row, column, index))
        .collect::<Result<Vec<Value>>>()
    })
    .collect::<Result<Vec<Vec<Value>>>>()?;
  response.row_count = Some(response.rows.len() as u64);

  return Ok(response);
}

macro_rules! convert_to_json_value {
  ($row: ident, $col_index: ident, $t:ty, $map: expr) => {{
    Ok(
      $row
        .get::<usize, Option<$t>>($col_index)
        .map_or_else(|| Value::Null, $map),
    )
  }};
}

// TODO(sagar): implement FromSql trait instead of doing this
fn get_json_value(
  row: &Row,
  column: &Column,
  col_index: usize,
) -> Result<Value, Error> {
  match column.type_() {
    &Type::BOOL => {
      convert_to_json_value!(row, col_index, bool, |v| Value::from(v))
    }
    &Type::INT4 => {
      convert_to_json_value!(row, col_index, i32, |v| Value::from(v))
    }
    &Type::INT8 => {
      convert_to_json_value!(row, col_index, i64, |v| Value::from(v))
    }
    &Type::FLOAT4 => {
      convert_to_json_value!(row, col_index, f32, |v| Value::from(v))
    }
    &Type::FLOAT8 => {
      convert_to_json_value!(row, col_index, f64, |v| Value::from(v))
    }
    &Type::TEXT | &Type::VARCHAR => {
      convert_to_json_value!(row, col_index, &str, |v| Value::from(v))
    }
    &Type::UUID => {
      convert_to_json_value!(row, col_index, Uuid, |v| Value::from(
        v.to_hyphenated().to_string()
      ))
    }
    &Type::JSONB | &Type::JSON_ARRAY => {
      convert_to_json_value!(row, col_index, Value, |v| v)
    }
    &Type::TIMESTAMPTZ => {
      convert_to_json_value!(
        row,
        col_index,
        chrono::DateTime<chrono::Utc>,
        |value| {
          Value::from(value.format("%Y-%m-%d %H:%M:%S.%f").to_string())
        }
      )
    }
    &Type::TIMESTAMP => {
      convert_to_json_value!(row, col_index, NaiveDateTime, |value| {
        Value::from(value.format("%Y-%m-%d %H:%M:%S.%f").to_string())
      })
    }
    &Type::FLOAT4_ARRAY => {
      convert_to_json_value!(row, col_index, Vec<f32>, |v| { Value::from(v) })
    }
    &Type::VOID => Ok(Value::Null),
    t => Err(anyhow!("UnsupportedDataTypeError: {}", t)),
  }
}

impl ToSql for Param {
  fn to_sql(
    &self,
    ty: &Type,
    out: &mut BytesMut,
  ) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync + 'static>> {
    if self.0 == Value::Null {
      return Ok(IsNull::Yes);
    }

    match *ty {
      Type::FLOAT4_ARRAY => match &self.0 {
        Value::String(value) => {
          let vector =
            serde_json::from_str::<Vec<f32>>(value).map_err(|err| {
              anyhow!("Error deserializing FLOAT4_ARRAY: {:?}", err)
            })?;
          return vector.to_sql(ty, out);
        }
        _ => {}
      },
      _ => {}
    };

    match &self.0 {
      Value::Bool(v) => {
        out.put_i8(if *v { 1 } else { 0 });
        Ok(IsNull::No)
      }
      Value::Number(v) => match *ty {
        Type::INT4 => {
          out.put_i32(v.as_i64().unwrap().try_into().unwrap());
          Ok(IsNull::No)
        }
        Type::INT8 => {
          out.put_i64(v.as_i64().unwrap());
          Ok(IsNull::No)
        }
        Type::FLOAT4 => {
          out.put_f32(v.as_f64().unwrap() as f32);
          Ok(IsNull::No)
        }
        Type::FLOAT8 => {
          out.put_f64(v.as_f64().unwrap());
          Ok(IsNull::No)
        }
        _ => {
          Err(anyhow!("to_sql: unsupported number type - [ {} ]", ty).into())
        }
      },
      Value::Object(v) => json!(v).to_sql(ty, out),
      Value::Array(v) => json!(v).to_sql(ty, out),
      Value::String(v) => match *ty {
        Type::TIMESTAMPTZ | Type::TIMESTAMP => {
          let date = chrono::DateTime::parse_from_rfc3339(&v)?;
          date.to_sql(ty, out)?;
          Ok(IsNull::No)
        }
        Type::VARCHAR
        | Type::TEXT
        | Type::BPCHAR
        | Type::NAME
        | Type::UNKNOWN
        | Type::JSONB
        | Type::JSON_ARRAY => {
          // Note(sagar): this is what serde_json does
          if *ty == Type::JSONB {
            out.put_u8(1);
          }
          out.write_str(&v)?;
          Ok(IsNull::No)
        }
        _ => Err(format!("to_sql: unsupported type - [ {} ]", ty).into()),
      },
      _ => Err(format!("to_sql: unsupported type - [ {} ]", ty).into()),
    }
  }

  fn accepts(ty: &Type) -> bool {
    match *ty {
      Type::BOOL
      | Type::INT4
      | Type::INT8
      | Type::VARCHAR
      | Type::TEXT
      | Type::TIMESTAMP
      | Type::TIMESTAMPTZ
      | Type::FLOAT4
      | Type::FLOAT8
      | Type::FLOAT4_ARRAY
      | Type::JSONB
      | Type::JSON_ARRAY => true,
      _ => {
        error!("Unsupported type: {}", ty);
        false
      }
    }
  }

  fn to_sql_checked(
    &self,
    ty: &Type,
    out: &mut BytesMut,
  ) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync + 'static>> {
    self.to_sql(ty, out)
  }
}

#[cfg(test)]
mod tests {
  // #[tokio::test]
  // async fn postgres_test() {
  // use super::{create_connection, Param};
  // use crate::extensions::postgres::postgres::execute_query;
  // use serde_json::Value;
  // let (client, connection) = create_connection(&format!(
  //   "postgresql://{}:{}@{}:{}/{}",
  //   "{user}", "{password}", "{host}", 5432, "{databas}"
  // ))
  // .await
  // .unwrap();

  // let handle = tokio::spawn(async {
  //   if let Err(e) = connection.await {
  //     tracing::error!("connection error: {}", e);
  //   }
  // });

  // let res = execute_query(
  //   &client,
  //   "SELECT 1",
  //   &vec![],
  // )
  // .await
  // .unwrap();
  // }
}
