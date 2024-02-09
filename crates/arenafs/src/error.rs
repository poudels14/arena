use thiserror::Error;

#[allow(unused)]
#[derive(Error, Debug)]
pub enum Error {
  #[error("data backend error")]
  BackendError(#[from] sqlx::Error),

  #[error("data backend disconnected")]
  BackendDisconnected(String),

  #[error("unknown error")]
  Unknown,
}
