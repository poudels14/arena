use anyhow::Result;
use sqlx::{FromRow, Pool, Postgres};

/// Dqs node
#[derive(FromRow, Debug, Clone)]
pub struct DqsNode {
  pub id: String,
  pub host: String,
  pub port: i32,
  pub status: String,
}

pub async fn delete_dqs_node_with_id(
  pool: &Pool<Postgres>,
  id: &str,
) -> Result<()> {
  sqlx::query("DELETE FROM app_clusters WHERE id = $1")
    .bind(&id)
    .execute(pool)
    .await?;
  Ok(())
}

pub async fn update_dqs_node_status_with_id(
  pool: &Pool<Postgres>,
  id: &str,
  status: &str,
) -> Result<()> {
  sqlx::query("UPDATE app_clusters SET status = $1 WHERE id = $2")
    .bind(status)
    .bind(&id)
    .execute(pool)
    .await?;
  Ok(())
}

pub async fn get_dqs_node_with_id(
  pool: &Pool<Postgres>,
  id: &str,
) -> Result<Option<DqsNode>> {
  let deployment = sqlx::query_as("SELECT * FROM app_clusters WHERE id = $1")
    .bind(&id)
    .fetch_optional(pool)
    .await?;
  Ok(deployment)
}

pub async fn insert_dqs_node(
  pool: &Pool<Postgres>,
  node: &DqsNode,
) -> Result<()> {
  sqlx::query(
    r#"INSERT INTO app_clusters
    (id, host, port, status)
    VALUES ($1, $2, $3, $4)
    "#,
  )
  .bind(&node.id)
  .bind(&node.host)
  .bind(&node.port)
  .bind(&node.status)
  .execute(pool)
  .await?;
  Ok(())
}
