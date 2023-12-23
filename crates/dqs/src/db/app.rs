#[allow(unused)]
pub use apps::table;
pub use diesel::prelude::*;
use serde_json::Value;
use std::time::SystemTime;

#[derive(Queryable, Debug, Clone)]
pub struct App {
  pub id: String,
  pub workspace_id: String,
  pub template: Option<Value>,
  pub archived_at: Option<SystemTime>,
}

diesel::table! {
  apps (id) {
    id -> Varchar,
    workspace_id -> Varchar,
    template -> Nullable<Jsonb>,
    archived_at -> Nullable<Timestamp>,
  }
}
