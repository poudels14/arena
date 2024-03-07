use serde_json::Value;
use sqlx::types::chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(FromRow, Debug, Clone)]
pub struct Workspace {
  pub id: String,
  pub name: String,
  pub config: Value,
  pub archived_at: Option<NaiveDateTime>,
}
