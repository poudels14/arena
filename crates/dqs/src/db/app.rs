pub use apps::table;
pub use diesel::prelude::*;
use std::time::SystemTime;

#[derive(Queryable, Debug, Clone)]
pub struct App {
  pub id: String,
  pub workspace_id: String,
  pub archived_at: Option<SystemTime>,
}

diesel::table! {
  apps (id) {
    id -> Varchar,
    workspace_id -> Varchar,
    archived_at -> Nullable<Timestamp>,
  }
}
