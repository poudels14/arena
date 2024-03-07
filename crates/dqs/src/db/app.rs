use serde_json::Value;
use sqlx::types::chrono::NaiveDate;
use sqlx::FromRow;

#[derive(FromRow, Debug, Clone)]
pub struct App {
  pub id: String,
  pub workspace_id: String,
  pub template: Option<Value>,
  pub owner_id: Option<String>,
  pub archived_at: Option<NaiveDate>,
}
