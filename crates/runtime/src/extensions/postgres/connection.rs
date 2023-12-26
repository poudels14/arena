use std::sync::Once;

use anyhow::Result;
use openssl::ssl::SslConnector;
use openssl::ssl::SslMethod;
use openssl::x509::X509;
use postgres::Socket;
use postgres_openssl::MakeTlsConnector;
use postgres_openssl::TlsStream;
use tokio_postgres::tls::NoTlsStream;
use tokio_postgres::Client;
use tokio_postgres::Connection as PostgresConnection;
use tokio_postgres::NoTls;

static INIT_PEMS: Once = Once::new();
static mut PEMS: Option<Vec<X509>> = None;

pub enum Connection {
  Tls {
    connection: PostgresConnection<Socket, TlsStream<Socket>>,
  },
  NoTls {
    connection: PostgresConnection<Socket, NoTlsStream>,
  },
}

impl Connection {
  pub async fn listen(self) -> Result<(), postgres::Error> {
    match self {
      Self::Tls { connection } => connection.await,
      Self::NoTls { connection } => connection.await,
    }
  }
}

pub async fn create_connection(
  db_url: &str,
  ssl: bool,
) -> Result<(Client, Connection)> {
  // TODO(sagar): permission checks
  match ssl {
    true => {
      INIT_PEMS.call_once(|| unsafe {
        PEMS = Some(vec![X509::from_pem(include_bytes!(
          "./certs/aws-global-bundle.pem"
        ))
        .unwrap()])
      });

      let mut builder = SslConnector::builder(SslMethod::tls_client())?;
      unsafe {
        PEMS.as_ref().map(|pems| {
          pems.iter().for_each(|pem| {
            builder.cert_store_mut().add_cert(pem.clone()).unwrap();
          })
        })
      };
      let connector = MakeTlsConnector::new(builder.build());

      let (client, connection) =
        tokio_postgres::connect(db_url, connector).await?;

      Ok((client, Connection::Tls { connection }))
    }
    false => {
      let (client, connection) = tokio_postgres::connect(db_url, NoTls).await?;
      Ok((client, Connection::NoTls { connection }))
    }
  }
}
