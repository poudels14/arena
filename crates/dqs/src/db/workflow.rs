use diesel::prelude::*;
use serde_json::Value;
use std::time::SystemTime;

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
pub struct WorkflowRun {
  pub id: String,
  pub workspace_id: String,
  pub parent_app_id: Option<String>,
  pub template: Value,
  pub config: Value,
  pub state: Value,
  pub status: String,
  pub triggered_by: Value,
  pub triggered_at: SystemTime,
  pub last_heartbeat_at: Option<SystemTime>,
}

diesel::table! {
  workflow_runs (id) {
    id -> Varchar,
    workspace_id -> Varchar,
    parent_app_id -> Nullable<Varchar>,
    template -> Jsonb,
    config -> Jsonb,
    state -> Jsonb,
    status -> Varchar,
    triggered_by -> Jsonb,
    triggered_at -> Timestamp,
    last_heartbeat_at -> Nullable<Timestamp>,
  }
}
