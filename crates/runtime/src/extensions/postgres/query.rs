use std::fmt::Write;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use bytes::BufMut;
use bytes::BytesMut;
use deno_core::serde_json::{json, Value};
use futures::TryStreamExt;
use postgres::types::ToSql;
use postgres::types::Type;
use serde::{Deserialize, Serialize};
use tokio_postgres::types::IsNull;
use tokio_postgres::Client;
use tokio_postgres::SimpleQueryMessage;
use tokio_postgres::{Column, Row};
use tracing::error;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize)]
pub struct Param(Value);

#[derive(Clone, Debug, Serialize)]
pub struct QueryResponse {
  pub columns: Vec<String>,

  pub rows: Vec<Vec<Value>>,

  /// Only set if the query returns rows
  pub row_count: Option<u64>,

  pub modified_rows: Option<u64>,
}

/**
 * Returns a tuple of (columns, rows) where the order of the values
 * in each row is same as the order of `columns`.
 */
#[tracing::instrument(skip(client, params), level = "debug")]
pub async fn execute_query(
  client: &Client,
  query: &str,
  params: &Option<Vec<Param>>,
) -> Result<QueryResponse, Error> {
  // If there are no params, execute it as simple query
  if params.as_ref().map(|p| p.is_empty()).unwrap_or(true) {
    return execute_simple_query(client, query).await;
  }

  let mut response = QueryResponse {
    columns: vec![],
    rows: vec![],
    row_count: None,
    modified_rows: None,
  };
  let res: Vec<Row> =
    match client.query_raw(query, params.as_ref().unwrap()).await {
      // TODO: stream the response?
      Ok(stream) => match stream.try_collect().await {
        Ok(data) => Ok::<Vec<Row>, anyhow::Error>(data),
        Err(err) => bail!("Error reading rows stream: {}", err.to_string()),
      },
      Err(err) => {
        bail!("Error executing query: {}", err.to_string())
      }
    }?;

  response.columns = res
    .get(0)
    .map(|row| row.columns().iter().map(|c| c.name().to_string()).collect())
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

async fn execute_simple_query(
  client: &Client,
  query: &str,
) -> Result<QueryResponse> {
  let mut response = QueryResponse {
    columns: vec![],
    rows: vec![],
    row_count: None,
    modified_rows: None,
  };

  let simple_response = client
    .simple_query(query)
    .await
    .context("Error executing simple query")?;

  let mut columns: Option<Vec<String>> = None;
  response.rows = simple_response
    .iter()
    .map(|message| match message {
      SimpleQueryMessage::Row(row) => {
        if columns.is_none() {
          response.row_count = Some(simple_response.len() as u64);
          columns =
            Some(row.columns().iter().map(|c| c.name().to_string()).collect());
        }

        row
          .columns()
          .iter()
          .enumerate()
          .map(|(index, _)| Value::from(row.get(index)))
          .collect()
      }
      SimpleQueryMessage::CommandComplete(len) => {
        response.modified_rows = Some(*len);
        vec![]
      }
      _ => unimplemented!(),
    })
    .collect::<Vec<Vec<Value>>>();

  Ok(response)
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
        |v| { Value::from(v.to_rfc3339()) }
      )
    }
    &Type::TIMESTAMP => {
      convert_to_json_value!(row, col_index, chrono::NaiveDateTime, |v| {
        Value::from(v.to_string())
      })
    }
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
        _ => {
          Err(anyhow!("to_sql: unsupported number type - [ {} ]", ty).into())
        }
      },
      Value::Object(v) => {
        json!(v).to_sql(ty, out)?;
        Ok(IsNull::No)
      }
      Value::Array(v) => {
        json!(v).to_sql(ty, out)?;
        Ok(IsNull::No)
      }

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
