use sqlx::types::chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(FromRow, Debug, Clone)]
pub struct EnvVar {
  pub id: String,
  pub workspace_id: Option<String>,
  pub name: String,
  pub app_template_id: Option<String>,
  pub app_id: Option<String>,
  pub key: String,
  pub value: String,
  pub archived_at: Option<NaiveDateTime>,
}
