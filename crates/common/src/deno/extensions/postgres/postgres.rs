use super::tls;
use super::tls::RustlsStream;
use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use bytes::BufMut;
use bytes::BytesMut;
use deno_core::serde_json::{json, Value};
use futures::TryStreamExt;
use postgres::types::ToSql;
use postgres::types::Type;
use postgres::Socket;
use rustls::{OwnedTrustAnchor, RootCertStore};
use rustls_pemfile::read_all;
use serde::Deserialize;
use std::fmt::Write;
use std::io::Cursor;
use std::sync::Arc;
use std::sync::Once;
use tokio_postgres::types::IsNull;
use tokio_postgres::Client;
use tokio_postgres::Connection;
use tokio_postgres::{Column, Row};
use tracing::error;
use uuid::Uuid;

static INIT_CERTS: Once = Once::new();
static mut CERTS: Option<rustls::ClientConfig> = None;

#[derive(Default, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryOptions {
  /// Whether to update column names to camel case
  pub camel_case: bool,
}

pub async fn create_connection(
  db_url: &str,
) -> Result<(Client, Connection<Socket, RustlsStream<Socket>>)> {
  INIT_CERTS.call_once(|| unsafe {
    CERTS = Some(get_tls_config());
  });

  let tls = tls::MakeRustlsConnect::new(unsafe { CERTS.clone().unwrap() });
  // TODO(sagar): permission checks
  let (client, connection) = tokio_postgres::connect(db_url, tls).await?;
  Ok((client, connection))
}

/**
 * Returns a tuple of (columns, rows) where the order of the values
 * in each row is same as the order of `columns`.
 */
pub async fn execute_query(
  client: &Client,
  query: &str,
  params: &Vec<Param>,
) -> Result<(Vec<String>, Vec<Vec<Value>>), Error> {
  // TODO(sagar): don't need this once JS prints the error properly
  let res: Vec<Row> = match client.query_raw(query, params).await {
    Ok(stream) => match stream.try_collect().await {
      Ok(data) => Ok(data),
      Err(e) => {
        error!("Error: {}", e);
        Err(e)
      }
    },
    Err(e) => {
      error!("Error: {}", e);
      Err(e)
    }
  }?;

  let mut cols = None;
  let rows: Vec<Vec<Value>> = res
    .iter()
    .map(|r: &Row| {
      if cols.is_none() {
        cols = Some(r.columns().iter().map(|c| c.name().to_string()).collect());
      }
      r.columns()
        .iter()
        .map(|c| get_json_value(c, r))
        .collect::<Result<Vec<Value>>>()
    })
    .collect::<Result<Vec<Vec<Value>>>>()?;

  return Ok((cols.unwrap_or_default(), rows));
}

macro_rules! convert_to_json_value {
  ($row: ident, $col: ident, $t:ty, $map: expr) => {{
    Ok(
      $row
        .get::<&str, Option<$t>>($col.name())
        .map_or_else(|| Value::Null, $map),
    )
  }};
}

// TODO(sagar): implement FromSql trait instead of doing this
fn get_json_value(column: &Column, row: &Row) -> Result<Value, Error> {
  match column.type_() {
    &Type::BOOL => {
      convert_to_json_value!(row, column, bool, |v| Value::from(v))
    }
    &Type::INT4 => convert_to_json_value!(row, column, i32, |v| Value::from(v)),
    &Type::INT8 => convert_to_json_value!(row, column, i64, |v| Value::from(v)),
    &Type::TEXT | &Type::VARCHAR => {
      convert_to_json_value!(row, column, &str, |v| Value::from(v))
    }
    &Type::UUID => convert_to_json_value!(row, column, Uuid, |v| Value::from(
      v.to_hyphenated().to_string()
    )),

    &Type::JSONB | &Type::JSON_ARRAY => {
      convert_to_json_value!(row, column, Value, |v| v)
    }
    &Type::TIMESTAMPTZ => {
      convert_to_json_value!(row, column, chrono::DateTime<chrono::Utc>, |v| {
        Value::from(v.to_rfc3339())
      })
    }
    &Type::TIMESTAMP => {
      convert_to_json_value!(row, column, chrono::NaiveDateTime, |v| {
        Value::from(v.to_string())
      })
    }
    t => Err(anyhow!("UnsupportedDataTypeError: {}", t)),
  }
}

fn get_tls_config() -> rustls::ClientConfig {
  let mut root_store = RootCertStore::empty();
  root_store.add_server_trust_anchors(
    webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
      OwnedTrustAnchor::from_subject_spki_name_constraints(
        ta.subject,
        ta.spki,
        ta.name_constraints,
      )
    }),
  );

  let mut cert_reader =
    Cursor::new(include_str!("certs/aws-global-bundle.pem"));
  let pem_certs = read_all(&mut cert_reader).unwrap();

  for item in pem_certs.iter() {
    match item {
      rustls_pemfile::Item::X509Certificate(v) => {
        root_store.add(&rustls::Certificate(v.to_vec())).unwrap();
      }
      _ => {}
    }
  }

  for cert in rustls_native_certs::load_native_certs()
    .expect("could not load platform certs")
  {
    root_store.add(&rustls::Certificate(cert.0)).unwrap();
  }

  let mut config = rustls::ClientConfig::builder()
    .with_safe_defaults()
    .with_root_certificates(root_store)
    .with_no_client_auth();

  // TODO(sagar): support verify-ca/verify-full ssl modes
  config
    .dangerous()
    .set_certificate_verifier(Arc::new(tls::AcceptAllVerifier {}));

  config
}

#[derive(Clone, Debug, Deserialize)]
pub struct Param(Value);

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
