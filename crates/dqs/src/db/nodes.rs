pub use diesel::prelude::*;

/// Dqs node
#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = app_clusters)]
pub struct DqsNode {
  pub id: String,
  pub host: String,
  pub port: i32,
  pub status: String,
}

diesel::table! {
  app_clusters (id) {
    id -> Varchar,
    host -> Varchar,
    port -> Integer,
    status -> Varchar,
    started_at -> Timestamp,
  }
}
