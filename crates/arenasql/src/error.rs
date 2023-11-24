use std::fmt;

use datafusion::error::DataFusionError;
use sqlparser::parser;

#[derive(Debug, Clone)]
pub enum Error {
  InvalidQuery(String),
  UnsupportedQuery(&'static str),
  InvalidOperation(String),
  ParserError(String),
  TransactionFinished,
  StorageError(String),
  SerdeError(&'static str),
  ExecutionError(String),
  SystemError(&'static str),
  DataFusionError(String),
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
    Self::StorageError(e.into_string())
  }
}

impl From<DataFusionError> for Error {
  fn from(e: DataFusionError) -> Self {
    Self::DataFusionError(e.to_string())
  }
}
