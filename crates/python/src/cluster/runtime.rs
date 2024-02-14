use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::{anyhow, Result};
use arenafs::{FileSystem, FilesCache, MountOption};
use derivative::Derivative;
use serde::Serialize;
use tokio::net::UnixStream;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;

use crate::fs::NoopBackend;
use crate::grpc::python_runtime_client::PythonRuntimeClient;
use crate::grpc::{self, ExecCodeResponse};
use crate::utils::NANOID_CHARS;

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct Runtime {
  pub id: String,
  /// API key needed to access the runtime
  pub api_key: String,

  /// Unix socket file
  socket: String,

  #[derivative(Debug = "ignore")]
  files_cache: Arc<Mutex<FilesCache>>,
}

#[derive(Serialize)]
pub struct File {
  id: String,
  parent_id: Option<String>,
  path: String,
  /// length of content bytes before encoding to base64
  size: usize,
  /// base64 encoded file content
  content: String,
}

pub async fn init(socket_file: &str) -> Result<Runtime> {
  let id = nanoid::nanoid!(22, &NANOID_CHARS);
  let api_key = nanoid::nanoid!(49, &NANOID_CHARS);
  let runtime = Runtime {
    id,
    api_key,
    socket: socket_file.to_owned(),
    files_cache: Arc::new(Mutex::new(FilesCache::new())),
  };

  Ok(runtime)
}

impl Runtime {
  pub async fn mount_fs(&self, path: String) -> Result<()> {
    let filesystem = FileSystem::with_backend(
      arenafs::Options {
        root_id: None,
        user_id: 1000,
        group_id: 1000,
      },
      self.files_cache.clone(),
      Arc::new(NoopBackend {}),
    )
    .await
    .unwrap();

    let options = vec![
      MountOption::RW,
      MountOption::FSName("arenafs".to_string()),
      MountOption::Suid,
      MountOption::AutoUnmount,
    ];

    std::thread::spawn(move || {
      filesystem.mount(&path, &options).unwrap();
    });

    Ok(())
  }

  pub async fn exec_code(&self, code: String) -> Result<ExecCodeResponse> {
    let socket = self.socket.clone();
    let channel = Endpoint::try_from("http://localhost")?
      .connect_with_connector(service_fn(move |_: Uri| {
        UnixStream::connect(socket.clone())
      }))
      .await?;

    let mut client = PythonRuntimeClient::new(channel);
    let req = tonic::Request::new(grpc::ExecCodeRequest { code });

    let response = client.exec_code(req).await?;
    Ok(response.into_inner())
  }

  pub fn list_files_updated_after(&self, since: Instant) -> Result<Vec<File>> {
    let new_files = self
      .files_cache
      .lock()
      .map_err(|e| anyhow!("Error getting file cache: {:?}", e))?
      .list_files_updated_after(since);
    let files = new_files
      .into_iter()
      .map(|file| File {
        id: file.id,
        parent_id: file.parent_id,
        path: file.path,
        size: file.content.len(),
        content: base64::encode(file.content),
      })
      .collect();

    Ok(files)
  }

  pub async fn terminate(&self) -> Result<()> {
    Ok(())
  }
}
