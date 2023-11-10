mod database;

use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, bail, Result};
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use dashmap::DashMap;
use libsql::Params;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use self::database::{Column, SqliteDatabase};

#[derive(Clone)]
pub struct State {
  /// Store db instance of active transactions
  pub transactions: Arc<DashMap<String, SqliteDatabase>>,
}

pub fn sqlite_router() -> Router {
  let state = State {
    transactions: Arc::new(DashMap::new()),
  };

  Router::new().route("/query", post(query)).with_state(state)
}

#[derive(Debug, Serialize, Deserialize)]
struct QueryRequest {
  path: String,
  stmt: String,
  params: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QueryResult {
  columns: Vec<Column>,
  rows: Vec<Vec<Value>>,
}

async fn query(
  Json(request): Json<QueryRequest>,
) -> Result<Json<QueryResult>, (StatusCode, String)> {
  ensure_db_path(&request.path)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

  let db = database::open(&request.path)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

  let (columns, rows) = db
    .query(&request.stmt, values_to_params(request.params).unwrap())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

  Ok(Json(QueryResult { columns, rows }))
}

fn ensure_db_path(path: &str) -> Result<()> {
  let db_path = Path::new(path);
  if let Some(parent) = db_path.parent() {
    if !parent.exists() {
      std::fs::create_dir_all(parent)
        .map_err(|_| anyhow!("Failed to create new database path"))?;
    }
  }

  Ok(())
}

fn values_to_params(values: Vec<Value>) -> Result<Params> {
  let mut params = Vec::with_capacity(values.len());
  for param in values {
    let value = match param {
      Value::Number(v) => {
        if let Some(v) = v.as_i64() {
          libsql::Value::Integer(v)
        } else {
          libsql::Value::Real(
            v.as_f64()
              .ok_or(anyhow!("Failed to convert param to float"))?,
          )
        }
      }
      Value::String(v) => libsql::Value::Text(v),
      Value::Null => libsql::Value::Null,
      _ => bail!("Failed to convert JSON type to Sqlite param"),
    };
    params.push(value);
  }

  Ok(Params::Positional(params))
}
