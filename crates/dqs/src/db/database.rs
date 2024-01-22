use diesel::prelude::*;
use serde_json::Value;

#[derive(Queryable, Debug, Clone)]
pub struct Database {
  pub id: String,
  pub workspace_id: String,
  pub app_id: Option<String>,
  pub credentials: Option<Value>,
  pub cluster_id: Option<String>,
}

#[derive(Queryable, Debug, Clone)]
pub struct DatabaseCluster {
  pub id: String,
  pub host: String,
  pub port: i32,
}

diesel::table! {
  databases (id) {
    id -> Varchar,
    workspace_id -> Varchar,
    app_id -> Nullable<Varchar>,
    credentials -> Nullable<Jsonb>,
    cluster_id -> Nullable<Varchar>,
  }
}

diesel::table! {
  database_clusters (id) {
    id -> Varchar,
    host -> Varchar,
    port -> Integer,
  }
}
