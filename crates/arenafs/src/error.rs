use thiserror::Error;

#[allow(unused)]
#[derive(Error, Debug)]
pub enum Error {
  #[error("data backend error")]
  BackendError(#[from] sqlx::Error),

  #[error("data backend disconnected")]
  BackendDisconnected(String),

  #[error("IO error")]
  IOError(#[from] std::io::Error),

  #[error("unknown error")]
  Unknown,
}
