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
  MultipleCommandsIntoPreparedStmt,
  StorageError,
  ArenaSqlError(arenasql::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Error: {:?}", self)
  }
}

macro_rules! user_error {
  ($severity:literal,$code:expr,$msg:expr) => {
    PgWireError::UserError(Box::new(ErrorInfo::new(
      $severity.to_owned(),
      $code.to_owned(),
      $msg,
    )))
  };
}

impl From<arenasql::Error> for Error {
  fn from(err: arenasql::Error) -> Self {
    Self::ArenaSqlError(err)
  }
}

impl From<Error> for PgWireError {
  fn from(value: Error) -> Self {
    match value {
      Error::InvalidConnection => user_error!(
        "FATAL",
        "08006",
        "System error [INVALID_CONNECTION]".to_owned()
      ),
      Error::CatalogNotFound(catalog) => user_error!(
        "FATAL",
        "3D000",
        format!("database \"{}\" does not exist", catalog)
      ),
      Error::MultipleCommandsIntoPreparedStmt => {
        user_error!(
          "ERROR",
          "42601",
          format!("cannot insert multiple commands into a prepared statement")
        )
      }
      Error::ArenaSqlError(err) => {
        user_error!("ERROR", err.code(), err.message())
      }
      _ => {
        user_error!("FATAL", "XX000", format!("System error [{:?}]", value))
      }
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
