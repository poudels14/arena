use std::fmt;

use datafusion::error::DataFusionError;
use sqlparser::parser;

use crate::schema::{Column, SerializedCell};

#[macro_export]
macro_rules! bail {
  ($err:expr) => {
    return Err($err);
  };
}

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
  NullConstraintViolated {
    table: String,
    column: String,
  },
  RelationAlreadyExists(String),
  RelationDoesntExist(String),
  SchemaDoesntExist(String),
  ColumnDoesntExist(String),
  UnsupportedQueryFilter(String),
  UnsupportedQuery(String),
  InvalidQuery(String),
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
      | Self::UnsupportedQueryFilter(_)
      | Self::UnsupportedQuery(_)
      | Self::InvalidQuery(_)
      | Self::NullConstraintViolated { .. }
      | Self::RelationAlreadyExists(_)
      | Self::RelationDoesntExist(_)
      | Self::SchemaDoesntExist(_)
      | Self::ColumnDoesntExist(_)
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
      | Self::UnsupportedQueryFilter(msg)
      | Self::UnsupportedQuery(msg)
      | Self::InvalidQuery(msg)
      | Self::IOError(msg)
      | Self::SerdeError(msg)
      | Self::InternalError(msg)
      | Self::InvalidTransactionState(msg)
      | Self::DataFusionError(msg) => msg.to_owned(),
      Self::UniqueConstaintViolated { constraint, .. } => format!(
        "duplicate key value violates unique constraint \"{}\"",
        constraint
      ),
      Self::NullConstraintViolated { table, column } => {
        format!(
          r#"null value in column "{}" of relation "{}" violates not-null constraint"#,
          column, table,
        )
      }
      Self::RelationAlreadyExists(rel) => {
        format!(r#"relation "{rel}" already exists"#)
      }
      Self::RelationDoesntExist(rel) => {
        format!(r#"relation "{rel}" does not exist"#)
      }
      Self::SchemaDoesntExist(schema) => {
        format!(r#"schema "{schema}" does not exist"#)
      }
      Self::ColumnDoesntExist(col) => {
        format!(r#"column "{col}" does not exist"#)
      }
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
macro_rules! df_error {
  ($err:expr) => {
    datafusion::error::DataFusionError::External(Box::new($err))
  };
}

pub fn null_constraint_violation(table: &str, column: &str) -> DataFusionError {
  DataFusionError::External(Box::new(Error::NullConstraintViolated {
    table: table.to_owned(),
    column: column.to_owned(),
  }))
}
