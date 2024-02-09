use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

use super::Backend;
use super::{DbAttribute, DbFile};
use crate::error::Error;

pub struct PostgresBackend {
  pool: Pool<Postgres>,
}

impl PostgresBackend {
  pub async fn init(connection_string: &str) -> Result<Self, Error> {
    let pool = PgPoolOptions::new()
      .max_connections(3)
      .connect(connection_string)
      .await?;
    Ok(Self { pool })
  }
}

#[async_trait::async_trait]
impl Backend for PostgresBackend {
  async fn fetch_node(
    &self,
    id: Option<&String>,
  ) -> Result<Option<DbAttribute>, Error> {
    let node: Option<DbAttribute> = sqlx::query_as(
      r#"SELECT id, name, parent_id, is_directory, size, created_at, updated_at
      FROM files
      WHERE id = $1 AND archived_at IS NULL
    "#,
    )
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
    let rows: Vec<DbAttribute> = sqlx::query_as(
      r#"SELECT id, name, parent_id, is_directory, size, created_at, updated_at
      FROM files
      WHERE parent_id = $1 AND archived_at IS NULL
    "#,
    )
    .bind(&id)
    .fetch_all(&self.pool)
    .await?;
    Ok(rows)
  }

  async fn read_file(&self, id: String) -> Result<Option<DbFile>, Error> {
    let file: Option<DbFile> = sqlx::query_as(
      r#"SELECT id, file, size, updated_at
      FROM files
      WHERE id = $1 AND archived_at IS NULL
    "#,
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(file)
  }

  // async fn add_file(
  //   &self,
  //   parent_id: Option<String>,
  //   filename: String,
  // ) -> Result<(), Error> {
  //   let file: Option<DbFile> = sqlx::query_as(
  //     r#"INSERT INTO files(id, name, parent_id, is_directory, metadata, created_by)
  //     VALUES ($1, $2, $3, $4, '{}', 'system')
  //   "#,
  //   )
  //   // id VARCHAR(50) UNIQUE NOT NULL,
  //   // name VARCHAR(250) NOT NULL,
  //   // description TEXT,
  //   // -- parent id is either parent directory id or parent file id
  //   // -- if this file was derived, set id of the original file
  //   // -- derived files are used to stored extracted text from
  //   // -- pdf, audio, etc
  //   // parent_id VARCHAR(50),
  //   // is_directory BOOL,
  //   // size INTEGER NOT NULL DEFAULT 0,
  //   // file FILE,
  //   // content_type VARCHAR(100) DEFAULT NULL,
  //   // metadata JSONB,
  //   // -- id of the user who created the file/directory
  //   // created_by VARCHAR(50),
  //   // created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  //   // updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  //   // archived_at TIMESTAMP DEFAULT NULL
  //   .bind(id)
  //   .fetch_optional(&self.pool)
  //   .await?;
  // }

  // async fn write_file(
  //   &self,
  //   _id: String,
  //   _content: &[u8],
  // ) -> Result<(), Error> {
  //   unimplemented!()
  // }
}
