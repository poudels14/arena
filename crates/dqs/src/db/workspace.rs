pub use diesel::prelude::*;
use serde_json::Value;
use std::time::SystemTime;
pub use workspaces::table;

#[derive(Queryable, Debug, Clone)]
pub struct Workspace {
  pub id: String,
  pub name: String,
  pub config: Value,
  pub archived_at: Option<SystemTime>,
}

diesel::table! {
  workspaces (id) {
    id -> Varchar,
    name -> Varchar,
    config ->Jsonb,
    archived_at -> Nullable<Timestamp>,
  }
}
