use anyhow::Context;
use anyhow::Result;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use std::env;
use std::time::Duration;

pub fn create_connection_pool() -> Result<Pool<ConnectionManager<PgConnection>>>
{
  let database_url =
    env::var("DATABASE_URL").context("Missing env variable: DATABASE_URL")?;

  let manager = ConnectionManager::<PgConnection>::new(database_url);
  Ok(
    Pool::builder()
      .idle_timeout(Some(Duration::from_secs(60)))
      .max_size(2)
      .connection_timeout(Duration::from_secs(10))
      .test_on_check_out(true)
      .build(manager)?,
  )
}
