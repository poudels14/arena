use std::fmt;

use datafusion::error::DataFusionError;
use sqlparser::parser;

use crate::schema::{Column, SerializedCell};

#[derive(Debug, Clone)]
pub enum Error {
  UnsupportedOperation(String),
  UnsupportedDataType(String),
  ParserError(String),
  InvalidTransactionState(String),
  UniqueConstaintViolated {
    // name of the unique index
    constraint: String,
    columns: Vec<Column>,
    data: Vec<SerializedCell<Vec<u8>>>,
  },
  IOError(String),
  SerdeError(String),
  InternalError(String),
  DataFusionError(String),
}

impl Error {
  /// PostgresSQL code
  /// https://www.postgresql.org/docs/current/errcodes-appendix.html
  pub fn code(&self) -> &'static str {
    match self {
      // syntax_error
      Self::ParserError(_) => "42601",
      // invalid_transaction_state
      Self::InvalidTransactionState(_) => "25000",
      // unique_violation
      Self::UniqueConstaintViolated { .. } => "23505",
      // internal_error
      Self::UnsupportedOperation(_)
      | Self::UnsupportedDataType(_)
      | Self::IOError(_)
      | Self::SerdeError(_)
      | Self::InternalError(_)
      | Self::DataFusionError(_) => "XX000",
    }
  }

  /// Error message
  pub fn message(&self) -> String {
    match self {
      Self::ParserError(msg)
      | Self::UnsupportedOperation(msg)
      | Self::UnsupportedDataType(msg)
      | Self::IOError(msg)
      | Self::SerdeError(msg)
      | Self::InternalError(msg)
      | Self::InvalidTransactionState(msg)
      | Self::DataFusionError(msg) => msg.to_owned(),
      Self::UniqueConstaintViolated { constraint, .. } => format!(
        "duplicate key value violates unique constraint \"{}\"",
        constraint
      ),
    }
  }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Error: {:?}", self)
  }
}

impl From<parser::ParserError> for Error {
  fn from(e: parser::ParserError) -> Self {
    Self::ParserError(e.to_string())
  }
}

impl From<rocksdb::Error> for Error {
  fn from(e: rocksdb::Error) -> Self {
    Self::IOError(e.into_string())
  }
}

impl From<DataFusionError> for Error {
  fn from(e: DataFusionError) -> Self {
    Self::DataFusionError(e.to_string())
  }
}

impl From<bincode::Error> for Error {
  fn from(e: bincode::Error) -> Self {
    Self::SerdeError(e.to_string())
  }
}

impl From<Error> for DataFusionError {
  fn from(err: Error) -> Self {
    DataFusionError::External(Box::new(err))
  }
}

#[macro_export]
macro_rules! df_execution_error {
    ($($arg:tt)*) => {
      datafusion::error::DataFusionError::Execution(format!($($arg)*))
    };
}

pub fn null_constraint_violation(table: &str, column: &str) -> DataFusionError {
  DataFusionError::Execution(format!(
    r#"null value in column "{}" of relation "{}" violates not-null constraint"#,
    column, table
  ))
}
