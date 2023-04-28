use super::tls;
use super::tls::RustlsStream;
use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use bytes::BufMut;
use bytes::BytesMut;
use futures::TryStreamExt;
use heck::ToLowerCamelCase;
use postgres::types::ToSql;
use postgres::types::Type;
use postgres::Socket;
use rustls;
use rustls::{OwnedTrustAnchor, RootCertStore};
use rustls_pemfile::read_all;
use serde::Deserialize;
use serde_json::json;
use serde_json::Map;
use serde_json::Value;
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

pub async fn execute_query(
  client: &Client,
  query: &str,
  params: &Vec<Param>,
) -> Result<Vec<Map<String, Value>>, Error> {
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

  let rows: Result<Vec<Map<String, Value>>, Error> = res
    .iter()
    .map(|r: &Row| {
      r.columns()
        .iter()
        .map(|c| {
          let value = get_json_value(c, r)?;
          Ok((String::from(c.name()).to_lower_camel_case(), value))
        })
        .collect()
    })
    .collect();

  return rows;
}

// TODO(sagar): implement FromSql trait instead of doing this
fn get_json_value(column: &Column, row: &Row) -> Result<Value, Error> {
  match column.type_() {
    &Type::BOOL => Ok(
      row
        .get::<&str, Option<bool>>(column.name())
        .map_or_else(|| Value::Null, |v| Value::from(v)),
    ),
    &Type::INT4 => Ok(
      row
        .get::<&str, Option<i32>>(column.name())
        .map_or_else(|| Value::Null, |v| Value::from(v)),
    ),
    &Type::INT8 => Ok(
      row
        .get::<&str, Option<i64>>(column.name())
        .map_or_else(|| Value::Null, |v| Value::from(v)),
    ),
    &Type::TEXT | &Type::VARCHAR => Ok(
      row
        .get::<&str, Option<&str>>(column.name())
        .map_or_else(|| Value::Null, |v| Value::from(v)),
    ),
    &Type::UUID => {
      Ok(row.get::<&str, Option<Uuid>>(column.name()).map_or_else(
        || Value::Null,
        |v| Value::from(v.to_hyphenated().to_string()),
      ))
    }

    &Type::JSONB | &Type::JSON_ARRAY => Ok(
      row
        .get::<&str, Option<Value>>(column.name())
        .map_or_else(|| Value::Null, |v| Value::from(v)),
    ),
    &Type::TIMESTAMPTZ => Ok(
      row
        .get::<&str, Option<chrono::DateTime<chrono::offset::Utc>>>(
          column.name(),
        )
        .map_or_else(|| Value::Null, |v| Value::from(v.to_string())),
    ),
    &Type::TIMESTAMP => Ok(
      row
        .get::<&str, Option<chrono::NaiveDateTime>>(column.name())
        .map_or_else(|| Value::Null, |v| Value::from(v.to_string())),
    ),
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

    match *ty {
      Type::BOOL => match self.0.as_bool() {
        Some(v) => {
          out.put_i8(if v { 1 } else { 0 });
          Ok(IsNull::No)
        }
        None => Err(
          format!("[expected type: {}, actual value: {}]", ty, self.0).into(),
        ),
      },
      Type::INT4 => match self.0.as_i64() {
        Some(v) => {
          out.put_i32(v.try_into().unwrap());
          Ok(IsNull::No)
        }
        None => Err(
          format!("[expected type: {}, actual value: {}]", ty, self.0).into(),
        ),
      },
      Type::INT8 => match self.0.as_i64() {
        Some(v) => {
          out.put_i64(v);
          Ok(IsNull::No)
        }
        None => Err(
          format!("[expected type: {}, actual value: {}]", ty, self.0).into(),
        ),
      },
      Type::VARCHAR
      | Type::TEXT
      | Type::BPCHAR
      | Type::NAME
      | Type::UNKNOWN => match self.0.as_str() {
        Some(v) => {
          out.write_str(v).unwrap();
          Ok(IsNull::No)
        }
        None => Err(
          format!("[expected type: {}, actual value: {}]", ty, self.0).into(),
        ),
      },
      Type::TIMESTAMPTZ | Type::TIMESTAMP => match self.0.as_str() {
        Some(v) => {
          let date = chrono::DateTime::parse_from_rfc3339(v)?;
          date.to_sql(ty, out)?;
          Ok(IsNull::No)
        }
        None => Err(
          format!("[expected type: {}, actual value: {}]", ty, self.0).into(),
        ),
      },
      Type::JSONB => match self.0.as_object() {
        Some(v) => {
          json!(v).to_sql(ty, out)?;
          Ok(IsNull::No)
        }
        None => Err(
          format!("[expected type: {}, actual value: {}]", ty, self.0).into(),
        ),
      },
      Type::JSON_ARRAY => match self.0.as_array() {
        Some(v) => {
          json!(v).to_sql(ty, out)?;
          Ok(IsNull::No)
        }
        None => Err(
          format!("[expected type: {}, actual value: {}]", ty, self.0).into(),
        ),
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
