use chrono::{Local, NaiveDateTime};
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

use super::Backend;
use super::{DbAttribute, DbFile};
use crate::error::Error;

pub struct PostgresBackend {
  pool: Pool<Postgres>,
  table_name: String,
  enable_write: bool,
}

impl PostgresBackend {
  pub async fn init(
    connection_string: &str,
    table_name: &str,
    enable_write: bool,
  ) -> Result<Self, Error> {
    let pool = PgPoolOptions::new()
      .max_connections(3)
      .connect(connection_string)
      .await?;
    Ok(Self {
      pool,
      table_name: table_name.to_owned(),
      enable_write,
    })
  }
}

#[async_trait::async_trait]
impl Backend for PostgresBackend {
  async fn fetch_node(
    &self,
    id: Option<&String>,
  ) -> Result<Option<DbAttribute>, Error> {
    let node: Option<DbAttribute> = sqlx::query_as(&format!(
      r#"SELECT id, name, parent_id, is_directory, size, created_at, updated_at
      FROM {0}
      WHERE id = $1 AND archived_at IS NULL
    "#,
      self.table_name
    ))
    .bind(&id)
    .fetch_optional(&self.pool)
    .await?;
    Ok(node)
  }

  /// Pass None to read root dir
  async fn fetch_children(
    &self,
    id: Option<&String>,
  ) -> Result<Vec<DbAttribute>, Error> {
    let rows: Vec<DbAttribute> = sqlx::query_as(&format!(
      r#"SELECT id, name, parent_id, is_directory, size, created_at, updated_at
      FROM {0}
      WHERE parent_id = $1 AND archived_at IS NULL
    "#,
      self.table_name
    ))
    .bind(&id)
    .fetch_all(&self.pool)
    .await?;
    Ok(rows)
  }

  async fn read_file(&self, id: String) -> Result<Option<DbFile>, Error> {
    let file: Option<DbFile> = sqlx::query_as(&format!(
      r#"SELECT id, file, size, updated_at
      FROM {0}
      WHERE id = $1 AND archived_at IS NULL
    "#,
      self.table_name
    ))
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(file)
  }

  async fn write_file(
    &self,
    attr: &DbAttribute,
    content: &[u8],
  ) -> Result<(), Error> {
    if !self.enable_write {
      return Ok(());
    }

    let now = Local::now().naive_utc();
    sqlx::query_as::<Postgres, DbAttribute>(&format!(
      r#"INSERT INTO files
      (id, name, parent_id, is_directory, size, file, content_type, metadata, created_at, created_by, updated_at)
      VALUES ($1, $2, $3, $4, $5, $6, 'test', '{}', $7,  'system', $7)
      "#,
      self.table_name
    ))
    .bind::<&str>(&attr.id.clone().unwrap())
    .bind::<&str>(&attr.name)
    .bind::<Option<&String>>(attr.parent_id.as_ref())
    .bind::<bool>(attr.is_directory)
    .bind::<i32>(attr.size)
    .bind::<&Value>(&json!({
      "content": base64::encode(&content)
    }))
    .bind::<NaiveDateTime>(now)
    .fetch_optional(&self.pool)
    .await?;
    Ok(())
  }
}
