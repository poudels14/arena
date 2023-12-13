use std::fmt;

use pgwire::error::{ErrorInfo, PgWireError};

pub type ArenaClusterError = Error;
pub type ArenaClusterResult<T> = Result<T, ArenaClusterError>;

#[derive(Debug, Clone)]
pub enum Error {
  /// Thrown if a session with same id as another existing session
  /// is created
  SessionAlreadyExists,
  CatalogNotFound(String),
  InvalidConnection,
  UnsupportedDataType(String),
  StorageError,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Error: {:?}", self)
  }
}

impl From<Error> for PgWireError {
  fn from(value: Error) -> Self {
    match value {
      Error::InvalidConnection => PgWireError::UserError(
        ErrorInfo::new(
          "FATAL".to_owned(),
          "08006".to_owned(),
          "System error [INVALID_CONNECTION]".to_owned(),
        )
        .into(),
      ),
      Error::CatalogNotFound(catalog) => PgWireError::UserError(
        ErrorInfo::new(
          "FATAL".to_owned(),
          "3D000".to_owned(),
          format!("database \"{}\" does not exist", catalog),
        )
        .into(),
      ),
      _ => PgWireError::UserError(
        ErrorInfo::new(
          "FATAL".to_owned(),
          "XX000".to_owned(),
          format!("System error [{:?}]", value),
        )
        .into(),
      ),
    }
  }
}

#[macro_export]
macro_rules! query_execution_error {
  ($message:expr) => {
    pgwire::error::PgWireError::UserError(
      pgwire::error::ErrorInfo::new(
        "ERROR".to_owned(),
        "XX000".to_owned(),
        $message,
      )
      .into(),
    )
  };
}
