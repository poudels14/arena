use anyhow::Result;
use sqlx::types::chrono::NaiveDateTime;
use sqlx::{FromRow, Pool, Postgres};

/// Dqs server deployment
#[derive(FromRow, Debug, Clone)]
pub struct Deployment {
  pub id: String,
  pub node_id: String,
  pub workspace_id: String,
  pub app_id: Option<String>,
  pub app_template_id: Option<String>,
  pub started_at: NaiveDateTime,
  pub last_heartbeat_at: Option<NaiveDateTime>,
  pub reboot_triggered_at: Option<NaiveDateTime>,
}

pub async fn delete_deployment_with_id(
  pool: &Pool<Postgres>,
  id: &str,
) -> Result<()> {
  sqlx::query("DELETE FROM app_deployments WHERE id = $1")
    .bind(&id)
    .execute(pool)
    .await?;
  Ok(())
}

pub async fn get_deployment_with_id(
  pool: &Pool<Postgres>,
  id: &str,
) -> Result<Option<Deployment>> {
  let deployment =
    sqlx::query_as("SELECT * FROM app_deployments WHERE id = $1")
      .bind(&id)
      .fetch_optional(pool)
      .await?;
  Ok(deployment)
}

pub async fn insert_deployment(
  pool: &Pool<Postgres>,
  deployment: &Deployment,
) -> Result<()> {
  sqlx::query(r#"INSERT INTO app_deployments
    (id, node_id, workspace_id, app_id, app_template_id, started_at, last_heartbeat_at, reboot_triggered_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    "#)
    .bind(&deployment.id)
    .bind(&deployment.node_id)
    .bind(&deployment.workspace_id)
    .bind(&deployment.app_id)
    .bind(&deployment.app_template_id)
    .bind(&deployment.started_at)
    .bind(&deployment.last_heartbeat_at)
    .bind(&deployment.reboot_triggered_at)
    .execute(pool)
    .await?;
  Ok(())
}
