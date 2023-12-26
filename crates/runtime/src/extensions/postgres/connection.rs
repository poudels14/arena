use std::sync::Once;

use anyhow::Result;
use openssl::ssl::SslConnector;
use openssl::ssl::SslMethod;
use openssl::x509::X509;
use postgres::Socket;
use postgres_openssl::MakeTlsConnector;
use postgres_openssl::TlsStream;
use serde::Deserialize;
use tokio_postgres::Client;
use tokio_postgres::Connection;

static INIT_PEMS: Once = Once::new();
static mut PEMS: Option<Vec<X509>> = None;

#[derive(Default, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryOptions {
  /// Whether to update column names to camel case
  pub camel_case: bool,
}

pub async fn create_connection(
  db_url: &str,
) -> Result<(Client, Connection<Socket, TlsStream<Socket>>)> {
  INIT_PEMS.call_once(|| unsafe {
    PEMS = Some(vec![X509::from_pem(include_bytes!(
      "./certs/aws-global-bundle.pem"
    ))
    .unwrap()])
  });

  let mut builder = SslConnector::builder(SslMethod::tls())?;
  unsafe {
    PEMS.as_ref().map(|pems| {
      pems.iter().for_each(|pem| {
        builder.cert_store_mut().add_cert(pem.clone()).unwrap();
      })
    })
  };
  let connector = MakeTlsConnector::new(builder.build());

  // TODO(sagar): permission checks
  let (client, connection) = tokio_postgres::connect(db_url, connector).await?;
  Ok((client, connection))
}
