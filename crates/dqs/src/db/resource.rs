use diesel::prelude::*;
pub use resources::table;
use serde_json::Value;
use std::time::SystemTime;

#[derive(Queryable, Debug, Clone)]
pub struct Resource {
  pub id: String,
  pub workspace_id: String,
  pub name: String,
  pub app_template_id: Option<String>,
  pub app_id: Option<String>,
  pub key: String,
  pub value: Value,
  pub ttype: String,
  pub secret: bool,
  pub context_id: Option<String>,
  pub archived_at: Option<SystemTime>,
}

diesel::table! {
  resources (id) {
    id -> Varchar,
    workspace_id -> Varchar,
    name -> Varchar,
    app_template_id -> Nullable<Varchar>,
    app_id -> Nullable<Varchar>,
    key -> Varchar,
    value -> Jsonb,
    #[sql_name = "type"]
    /// type
    ttype -> Varchar,
    secret -> Bool,
    context_id -> Nullable<Varchar>,
    archived_at -> Nullable<Timestamp>,
  }
}
