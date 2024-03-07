use serde_json::Value;
use sqlx::FromRow;

#[derive(FromRow, Debug)]
pub struct Widget {
  pub id: String,
  pub name: String,
  pub config: Value,
}
