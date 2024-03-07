use anyhow::Context;
use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;
use std::time::Duration;

pub async fn create_connection_pool() -> Result<Pool<Postgres>> {
  let database_url =
    env::var("DATABASE_URL").context("Missing env variable: DATABASE_URL")?;

  let max_pool_size = env::var("DATABASE_POOL_SIZE")
    .context("Missing env var: DATABASE_POOL_SIZE")
    .and_then(|s| {
      s.parse::<u32>()
        .context("Error parsing env variable: DATABASE_POOL_SIZE")
    })
    .unwrap_or(10);

  tracing::debug!("Connecting to database $DATABASE_URL");
  Ok(
    PgPoolOptions::new()
      .max_connections(max_pool_size)
      .idle_timeout(Some(Duration::from_secs(60)))
      .acquire_timeout(Duration::from_secs(30))
      .connect(&database_url)
      .await?,
  )
}
