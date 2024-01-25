use diesel::prelude::*;
pub use environment_variables::table;
use std::time::SystemTime;

#[derive(Queryable, Debug, Clone)]
pub struct EnvVar {
  pub id: String,
  pub workspace_id: Option<String>,
  pub name: String,
  pub app_template_id: Option<String>,
  pub app_id: Option<String>,
  pub key: String,
  pub value: String,
  pub archived_at: Option<SystemTime>,
}

diesel::table! {
  environment_variables (id) {
    id -> Varchar,
    workspace_id -> Nullable<Varchar>,
    name -> Varchar,
    app_template_id -> Nullable<Varchar>,
    app_id -> Nullable<Varchar>,
    key -> Varchar,
    value -> Text,
    archived_at -> Nullable<Timestamp>,
  }
}
