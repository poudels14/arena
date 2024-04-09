pub mod postgres;

use chrono::NaiveDateTime;
use serde::Deserialize;
use sqlx::prelude::FromRow;
use sqlx::types::Json;

use crate::error::Error;

#[allow(dead_code)]
#[derive(Debug, Default, Clone, FromRow)]
pub struct DbAttribute {
  pub id: Option<String>,
  pub name: String,
  pub parent_id: Option<String>,
  pub is_directory: bool,
  pub size: i32,
  pub created_at: NaiveDateTime,
  pub updated_at: NaiveDateTime,
}

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct DbFile {
  pub id: String,
  pub file: Json<DbFileContent>,
  pub size: i32,
  pub updated_at: NaiveDateTime,
}

#[derive(Debug, FromRow, Deserialize)]
pub struct DbFileContent {
  pub content: String,
}

#[async_trait::async_trait]
pub trait Backend: Send + Sync {
  /// id is None for root dir
  async fn fetch_node(
    &self,
    id: Option<&String>,
  ) -> Result<Option<DbAttribute>, Error>;

  async fn fetch_children(
    &self,
    id: Option<&String>,
  ) -> Result<Vec<DbAttribute>, Error>;

  async fn read_file(&self, id: String) -> Result<Option<DbFile>, Error>;

  async fn write_file(
    &self,
    attr: &DbAttribute,
    content: &[u8],
  ) -> Result<(), Error>;
}
