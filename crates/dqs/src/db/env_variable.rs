use diesel::prelude::*;
pub use env_variables::table;
use serde_json::Value;

#[derive(Queryable, Debug, Clone)]
pub struct EnvVariable {
  pub id: String,
  pub workspace_id: String,
  pub key: String,
  pub value: Value,
  pub ttype: String,
  pub context_id: Option<String>,
}

diesel::table! {
  env_variables (id) {
    id -> Varchar,
    workspace_id -> Varchar,
    key -> Varchar,
    value -> Jsonb,
    #[sql_name = "type"]
    /// type
    ttype -> Varchar,
    context_id -> Nullable<Varchar>,
  }
}
