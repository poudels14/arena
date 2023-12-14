use crate::faiss_ffi::NativeError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
  Native(NativeError),
  IOError(String),
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "")
  }
}

impl std::error::Error for Error {}

impl From<NativeError> for Error {
  fn from(e: NativeError) -> Self {
    Self::Native(e)
  }
}
