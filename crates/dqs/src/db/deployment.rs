#[allow(unused)]
pub use app_deployments::table;
pub use diesel::prelude::*;
use std::time::SystemTime;

/// Dqs server deployment
#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = app_deployments)]
pub struct Deployment {
  pub id: String,
  pub node_id: String,
  pub workspace_id: String,
  pub app_id: Option<String>,
  pub app_template_id: Option<String>,
  pub started_at: SystemTime,
  pub last_heartbeat_at: Option<SystemTime>,
  pub reboot_triggered_at: Option<SystemTime>,
}

diesel::table! {
  app_deployments (id) {
    id -> Varchar,
    node_id -> Varchar,
    workspace_id -> Varchar,
    app_id -> Nullable<Varchar>,
    app_template_id -> Nullable<Varchar>,
    started_at -> Timestamp,
    last_heartbeat_at -> Nullable<Timestamp>,
    reboot_triggered_at -> Nullable<Timestamp>,
  }
}
