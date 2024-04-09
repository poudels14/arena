use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use arenafs::{Backend, FileSystem, FilesCache, MountOption, PostgresBackend};
use derivative::Derivative;
use serde::Serialize;
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;

use crate::fs::NoopBackend;
use crate::grpc::python_runtime_client::PythonRuntimeClient;
use crate::grpc::{self, ExecCodeResponse};
use crate::utils::NANOID_CHARS;

use super::runtime_spec as spec;

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

unsafe impl Sync for Runtime {}
unsafe impl Send for Runtime {}

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
  pub async fn mount_fs(
    &self,
    path: String,
    fs: Option<spec::FileSystem>,
  ) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<bool>(1);
    let files_cache = self.files_cache.clone();
    files_cache
      .lock()
      .map_err(|_| anyhow!("File cache lock error"))?
      .reset();
    std::thread::spawn(move || {
      let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(1)
        .build()
        .unwrap();

      let guard = rt.enter();
      let filesystem = rt
        .block_on(async {
          let backend: Arc<dyn Backend> = match fs {
            Some(ref fs) => Arc::new(
              PostgresBackend::init(
                &fs.connection_string,
                &fs.table_name,
                fs.enable_write.unwrap_or(false),
              )
              .await?,
            ),
            None => Arc::new(NoopBackend {}),
          };

          FileSystem::with_backend(
            arenafs::Options {
              root_id: fs.and_then(|s| s.root.clone()),
              user_id: 1000,
              group_id: 1000,
            },
            files_cache,
            backend,
          )
          .await
        })
        .unwrap();

      let options = vec![
        MountOption::RW,
        MountOption::FSName("arenafs".to_string()),
        MountOption::Suid,
        MountOption::AutoUnmount,
      ];

      rt.spawn(async move {
        let _ = tx.send(true).await;
      });
      filesystem.mount(&path, &options).unwrap();
      drop(guard);
    });

    let _ = rx.recv().await;
    // allow some time to mount fs
    tokio::time::sleep(Duration::from_millis(20)).await;
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

  #[allow(dead_code)]
  pub async fn terminate(&self) -> Result<()> {
    Ok(())
  }
}
