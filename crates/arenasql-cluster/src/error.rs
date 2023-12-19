use std::fmt;
use std::sync::Arc;

use pgwire::error::{ErrorInfo, PgWireError};

pub type ArenaClusterError = Error;
pub type ArenaClusterResult<T> = Result<T, ArenaClusterError>;

#[derive(Debug, Clone)]
pub enum Error {
  UserDoesntExist(String),
  InvalidPassword,
  /// Thrown when error occurs during IO
  IOError(Arc<std::io::Error>),
  /// Thrown if a session with same id as another existing session
  /// is created
  SessionAlreadyExists,
  CatalogNotFound(String),
  InvalidConnection,
  UnsupportedDataType(String),
  MultipleCommandsIntoPreparedStmt,
  ArenaSqlError(arenasql::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Error: {:?}", self)
  }
}

impl Error {
  pub fn severity(&self) -> &'static str {
    match self {
      Self::UserDoesntExist(_)
      | Self::InvalidPassword
      | Self::CatalogNotFound(_)
      | Self::InvalidConnection
      | Self::SessionAlreadyExists => "FATAL",
      Self::IOError(_)
      | Self::UnsupportedDataType(_)
      | Self::MultipleCommandsIntoPreparedStmt
      | Self::ArenaSqlError(_) => "Error",
    }
  }

  pub fn code(&self) -> &'static str {
    match self {
      Self::UserDoesntExist(_) => "28000",
      Self::InvalidPassword => "28P01",
      Self::CatalogNotFound(_) => "3D000",
      // connection_failure
      Self::InvalidConnection | Self::SessionAlreadyExists => "08006",
      Self::ArenaSqlError(e) => e.code(),
      Self::MultipleCommandsIntoPreparedStmt => "42601",
      Self::IOError(_) | Self::UnsupportedDataType(_) => "XX000",
    }
  }

  pub fn message(&self) -> String {
    match self {
      Self::UserDoesntExist(user) => {
        format!("role \"{}\" does not exist", user)
      }
      Self::InvalidPassword => format!("invalid_password"),
      Self::CatalogNotFound(catalog) => {
        format!("database \"{}\" does not exist", catalog)
      }
      Self::ArenaSqlError(e) => e.message(),
      Self::IOError(_)
      | Self::UnsupportedDataType(_)
      | Self::MultipleCommandsIntoPreparedStmt => {
        format!("cannot insert multiple commands into a prepared statement")
      }
      Self::InvalidConnection | Self::SessionAlreadyExists => {
        format!("Connection error")
      }
    }
  }
}

impl From<arenasql::Error> for Error {
  fn from(err: arenasql::Error) -> Self {
    Self::ArenaSqlError(err)
  }
}

impl From<std::io::Error> for Error {
  fn from(err: std::io::Error) -> Self {
    Self::IOError(err.into())
  }
}

impl From<Error> for PgWireError {
  fn from(error: Error) -> Self {
    PgWireError::UserError(Box::new(ErrorInfo::new(
      error.severity().to_owned(),
      error.code().to_owned(),
      error.message(),
    )))
  }
}
