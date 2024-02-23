#[allow(unused)]
pub use acls::table;
pub use diesel::prelude::*;
use serde_json::Value;
use std::time::SystemTime;

#[derive(Queryable, Debug, Clone)]
pub struct Acl {
  pub id: String,
  pub workspace_id: String,
  pub user_id: String,
  pub app_id: Option<String>,
  pub resource_id: Option<String>,
  // READ, WRITE, UPDATE, DELETE
  // corresponds to: SELECT, INSERT, UPDATE, DELETE sql query
  // OWNER and ADMIN will give all access
  pub access: String,
  // metadata will contain following fields:
  //  - table: table name; `*` for all table access
  //  - filter: SQL filter; `*` for no filter
  pub metadata: Value,
  pub archived_at: Option<SystemTime>,
}

diesel::table! {
  acls (id) {
    id -> Varchar,
    workspace_id -> Varchar,
    user_id -> Varchar,
    app_id -> Nullable<Varchar>,
    resource_id -> Nullable<Varchar>,
    access -> Varchar,
    metadata -> Jsonb,
    archived_at -> Nullable<Timestamp>,
  }
}
