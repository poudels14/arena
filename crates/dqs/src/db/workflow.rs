use serde_json::Value;
use sqlx::types::chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(FromRow, Debug, Clone)]
pub struct WorkflowRun {
  pub id: String,
  pub workspace_id: String,
  pub parent_app_id: Option<String>,
  pub template: Value,
  pub config: Value,
  pub state: Value,
  pub status: String,
  pub triggered_by: Value,
  pub triggered_at: NaiveDateTime,
  pub last_heartbeat_at: Option<NaiveDateTime>,
}
