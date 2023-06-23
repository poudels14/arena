pub use acls::table;
pub use diesel::prelude::*;
use std::time::SystemTime;

#[derive(Queryable, Debug, Clone)]
pub struct Acl {
  pub id: String,
  pub workspace_id: String,
  pub user_id: String,
  pub app_id: Option<String>,
  pub path: Option<String>,
  pub resource_id: Option<String>,
  pub access: String,
  pub archived_at: Option<SystemTime>,
}

diesel::table! {
  acls (id) {
    id -> Varchar,
    workspace_id -> Varchar,
    user_id -> Varchar,
    app_id -> Nullable<Varchar>,
    path -> Nullable<Varchar>,
    resource_id -> Nullable<Varchar>,
    access -> Varchar,
    archived_at -> Nullable<Timestamp>,
  }
}
