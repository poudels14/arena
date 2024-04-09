use std::time::Instant;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::runtime::File;
use super::runtime_spec::{FileSystem, RuntimeImage};
use super::Cluster;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateRuntimeRequest {
  image: RuntimeImage,
  #[serde(default)]
  fs: Option<FileSystem>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateRuntimeResponse {
  id: String,
  // API key needed to access the runtime
  api_key: String,
}

#[tracing::instrument(skip_all, level = "debug")]
pub async fn create_runtime(
  State(cluster): State<Cluster>,
  Json(request): Json<CreateRuntimeRequest>,
) -> Result<Json<CreateRuntimeResponse>, StatusCode> {
  match request.image {
    RuntimeImage::Python { .. } | RuntimeImage::Python310 { .. } => {
      let runtime = cluster
        .create_new_runtime(&request.image)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

      runtime
        .mount_fs("./mnt".to_owned(), request.fs)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

      Ok(Json(CreateRuntimeResponse {
        id: runtime.id,
        api_key: runtime.api_key,
      }))
    }
  }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecCodeRequest {
  code: String,

  // if true, updated files will be included in the response
  include_artifacts: Option<bool>,

  /// If set, overrides the current file system mount
  #[serde(default)]
  fs: Option<FileSystem>,
}

#[derive(Serialize)]
pub struct ExecCodeResponse {
  success: bool,
  stdout: String,
  stderr: String,
  data: Value,
  error: Option<String>,
  // list of new files created by the python code
  artifacts: Option<Vec<File>>,
}

#[tracing::instrument(skip_all, level = "debug")]
pub async fn exec_code(
  Path(runtime_id): Path<String>,
  State(cluster): State<Cluster>,
  Json(request): Json<ExecCodeRequest>,
) -> Result<Json<ExecCodeResponse>, (StatusCode, String)> {
  let request_at = Instant::now();
  let runtime = cluster
    .get_runtime(&runtime_id)
    .ok_or_else(|| (StatusCode::NOT_FOUND, format!("runtime not found")))?;

  if request.fs.is_some() {
    runtime
      .mount_fs("./mnt".to_owned(), request.fs)
      .await
      .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, format!("IO error")))?;
  }

  let response = runtime.exec_code(request.code).await.map_err(|e| {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      format!("Error executing code: {:?}", e),
    )
  })?;

  let artifacts = match request.include_artifacts.unwrap_or(false) {
    true => {
      Some(runtime.list_files_updated_after(request_at).map_err(|e| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          format!("Error getting updated files: {:?}", e),
        )
      })?)
    }
    false => None,
  };
  Ok(Json(ExecCodeResponse {
    success: response.success,
    error: response.error,
    data: response
      .data
      .map(|d| {
        Ok(json!({
          "type": d.r#type,
          "value": serde_json::from_str::<Value>(&d.value)?
        }))
      })
      .transpose()
      .map_err(|e: anyhow::Error| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          format!("Error serializing result: {:?}", e),
        )
      })?
      .unwrap_or_default(),
    stdout: response.stdout,
    stderr: response.stderr,
    artifacts,
  }))
}
