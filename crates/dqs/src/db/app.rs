use sqlx::types::chrono::NaiveDate;
use sqlx::FromRow;

#[derive(FromRow, Debug, Clone)]
pub struct App {
  pub id: String,
  pub workspace_id: String,
  pub template_id: Option<String>,
  pub template_version: Option<String>,
  pub owner_id: Option<String>,
  pub archived_at: Option<NaiveDate>,
}
