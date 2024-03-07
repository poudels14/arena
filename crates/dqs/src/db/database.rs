use anyhow::Result;
use serde_json::Value;
use sqlx::{FromRow, Pool, Postgres};

#[derive(FromRow, Debug, Clone)]
pub struct Database {
  pub id: String,
  pub workspace_id: String,
  pub app_id: Option<String>,
  pub credentials: Option<Value>,
  pub cluster_id: Option<String>,
}

#[derive(FromRow, Debug, Clone)]
pub struct DatabaseCluster {
  pub id: String,
  pub host: String,
  pub port: i32,
}

pub async fn get_database_with_app_id(
  pool: &Pool<Postgres>,
  workspace_id: &str,
  app_id: &str,
) -> Result<Option<Database>> {
  let deployment = sqlx::query_as(
    "SELECT * FROM databases WHERE workspace_id = $1 AND app_id = $2",
  )
  .bind(&workspace_id)
  .bind(&app_id)
  .fetch_optional(pool)
  .await?;
  Ok(deployment)
}

pub async fn get_database_cluster_with_id(
  pool: &Pool<Postgres>,
  cluster_id: &str,
) -> Result<Option<DatabaseCluster>> {
  let deployment =
    sqlx::query_as("SELECT * FROM database_clusters WHERE id = $1")
      .bind(&cluster_id)
      .fetch_optional(pool)
      .await?;
  Ok(deployment)
}
