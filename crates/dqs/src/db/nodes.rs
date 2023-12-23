pub use diesel::prelude::*;

/// Dqs node
#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = dqs_nodes)]
pub struct DqsNode {
  pub id: String,
  pub host: String,
  pub port: i32,
  pub status: String,
}

diesel::table! {
  dqs_nodes (id) {
    id -> Varchar,
    host -> Varchar,
    port -> Integer,
    status -> Varchar,
    started_at -> Timestamp,
  }
}
