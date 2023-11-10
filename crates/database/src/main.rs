mod sql;

use axum::Router;
use std::env;
use std::net::SocketAddr;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::prelude::*;
use tracing_tree::HierarchicalLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let subscriber = tracing_subscriber::registry()
    .with(
      EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy(),
    )
    .with(
      HierarchicalLayer::default()
        .with_indent_amount(2)
        .with_thread_names(true),
    );
  tracing::subscriber::set_global_default(subscriber).unwrap();

  let host = env::var("HOST").unwrap_or("127.0.0.1".to_owned());
  let port = env::var("PORT")
    .ok()
    .and_then(|p: String| p.parse().ok())
    .unwrap_or(5321);

  let databases = Router::new().nest("/sqlite", sql::sqlite_router());

  let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
  tracing::info!("listening on {}", addr);
  axum::Server::bind(&addr)
    .serve(databases.into_make_service())
    .await
    .unwrap();

  Ok(())
}
