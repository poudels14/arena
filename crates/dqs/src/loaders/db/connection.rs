use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use std::env;

pub fn create_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
  let database_url =
    env::var("DATABASE_URL").expect("DATABASE_URL must be set");

  let manager = ConnectionManager::<PgConnection>::new(database_url);
  Pool::builder()
    .test_on_check_out(true)
    .build(manager)
    .expect("Could not build connection pool")
}
