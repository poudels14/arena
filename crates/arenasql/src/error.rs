use std::fmt;
use std::sync::Arc;

use datafusion::common::SchemaError;
use datafusion::error::DataFusionError;
use once_cell::sync::Lazy;
use pgwire::error::{ErrorInfo, PgWireError};
use regex::Regex;
use sqlparser::parser::{self, ParserError};

use crate::schema::{Column, OwnedSerializedCell};

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
  InvalidDataType(String),
  ParserError(String),
  InvalidTransactionState(String),
  UniqueConstaintViolated {
    // name of the unique index
    constraint: String,
    columns: Vec<Column>,
    data: Vec<OwnedSerializedCell>,
  },
  NullConstraintViolated {
    table: String,
    column: String,
  },
  DatabaseDoesntExist(String),
  DatabaseAlreadyExists(String),
  // relation = table or index
  RelationAlreadyExists(String),
  RelationDoesntExist(String),
  SchemaDoesntExist(String),
  ColumnDoesntExist(String),
  UnsupportedQueryFilter(String),
  UnsupportedQuery(String),
  InvalidQuery(String),
  InvalidParameter(String),
  InternalError(String),
  DatabaseClosed,
  IOError(String),
  EncodingError(Arc<prost::EncodeError>),
  DecodingError(Arc<prost::DecodeError>),
  SerdeError(Arc<bincode::Error>),
  InsufficientPrivilege,
  DataFusionError(Arc<DataFusionError>),
  ReservedWord(String),
}

const RE_TABLE_NOT_FOUND: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"table '(?P<rel>[\w.]+)' not found").unwrap());
const RE_TABLE_DOESNT_EXIST: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"Table '(?P<rel>[\w.]+)' doesn't exist").unwrap());
const RE_TABLE_ALREADY_EXISTS: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"Table '(?P<table>[\w.]+)' already exists").unwrap()
});
const RE_CATALOG_ALREADY_EXISTS: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"Catalog '(?P<catalog>[\w.]+)' already exists").unwrap()
});

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
      // insufficient_privilege
      Self::InsufficientPrivilege => "42501",
      Self::DatabaseDoesntExist(_) => "3D000",
      // undefined_table or index
      Self::RelationDoesntExist(_) => "42P01",
      // duplicate_table or duplicate index
      Self::RelationAlreadyExists(_) => "42P07",
      // undefined_column
      Self::ColumnDoesntExist(_) => "42703",
      // internal_error
      Self::UnsupportedOperation(_)
      | Self::UnsupportedDataType(_)
      | Self::InvalidDataType(_)
      | Self::UnsupportedQueryFilter(_)
      | Self::UnsupportedQuery(_)
      | Self::InvalidQuery(_)
      | Self::NullConstraintViolated { .. }
      | Self::DatabaseAlreadyExists(_)
      | Self::SchemaDoesntExist(_)
      | Self::IOError(_)
      | Self::DecodingError(_)
      | Self::EncodingError(_)
      | Self::SerdeError(_)
      | Self::InternalError(_)
      | Self::DatabaseClosed
      | Self::ReservedWord(_)
      | Self::InvalidParameter(_)
      | Self::DataFusionError(_) => "XX000",
    }
  }

  /// Error message
  pub fn message(&self) -> String {
    match self {
      Self::UnsupportedDataType(t) => {
        format!("Unsupported data type: {:?}", t)
      }
      Self::ParserError(msg)
      | Self::UnsupportedOperation(msg)
      | Self::InvalidDataType(msg)
      | Self::UnsupportedQueryFilter(msg)
      | Self::UnsupportedQuery(msg)
      | Self::InvalidQuery(msg)
      | Self::ReservedWord(msg)
      | Self::InvalidParameter(msg)
      | Self::InvalidTransactionState(msg) => msg.to_owned(),
      Self::InsufficientPrivilege => format!("permission denied"),
      Self::InternalError(msg) => {
        tracing::warn!("Internal error: {:?}", msg);
        format!("Internal error")
      }
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
      Self::DatabaseDoesntExist(db) => {
        format!(r#"database "{db}" doesn't exist"#)
      }
      Self::DatabaseAlreadyExists(db) => {
        format!(r#"database "{db}" already exists"#)
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
      Self::DatabaseClosed => format!(r#"database already closed"#),
      Self::IOError(_)
      | Self::SerdeError(_)
      | Self::DecodingError(_)
      | Self::EncodingError(_) => {
        tracing::warn!("Error: {:?}", self);
        format!("IO error")
      }
      Self::DataFusionError(df_err) => match df_err.as_ref() {
        DataFusionError::External(err) => {
          if let Some(arena_err) = err.downcast_ref::<Error>() {
            arena_err.message()
          } else {
            tracing::warn!("Error: {:?}", err);
            format!("Internal error")
          }
        }
        DataFusionError::Context(_, e) => match e.as_ref() {
          DataFusionError::External(err) => {
            if let Some(arena_err) = err.downcast_ref::<Error>() {
              arena_err.message()
            } else {
              tracing::warn!(
                "Unknown error at {}:{}: {:?}",
                file!(),
                line!(),
                err
              );
              format!("Internal error")
            }
          }
          err => {
            tracing::warn!(
              "Unknown error at {}:{}: {:?}",
              file!(),
              line!(),
              err
            );
            format!("Internal error")
          }
        },
        DataFusionError::Plan(msg) | DataFusionError::Execution(msg) => {
          if let Some(err) = Self::from_df_error(df_err.as_ref()) {
            return err.message();
          }
          if msg.contains("not yet supported") || msg.contains("not supported")
          {
            tracing::warn!(
              "Unsupported query error: {}:{}: {:?}",
              file!(),
              line!(),
              msg
            );
            format!("Unsupported query")
          }
          // table '...' not found"
          else if msg.contains("table") && msg.contains("not found") {
            msg.to_owned()
          } else {
            tracing::warn!(
              "Unknown query error: {}:{}: {:?}",
              file!(),
              line!(),
              msg
            );
            format!("Unknown error")
          }
        }
        err => {
          tracing::warn!("Unknown error at {}:{}: {:?}", file!(), line!(), err);
          format!("Internal error")
        }
      },
    }
  }

  fn from_df_error(err: &DataFusionError) -> Option<Self> {
    match err {
      DataFusionError::Plan(ref msg) | DataFusionError::Execution(ref msg) => {
        if let Some(capture) = RE_TABLE_NOT_FOUND.captures(msg) {
          Some(Self::RelationDoesntExist(capture["rel"].to_owned()))
        } else if let Some(capture) = RE_TABLE_DOESNT_EXIST.captures(msg) {
          Some(Self::RelationDoesntExist(capture["rel"].to_owned()))
        } else if let Some(capture) = RE_CATALOG_ALREADY_EXISTS.captures(msg) {
          Some(Self::DatabaseAlreadyExists(capture["catalog"].to_owned()))
        } else if let Some(capture) = RE_TABLE_ALREADY_EXISTS.captures(msg) {
          Some(Self::RelationAlreadyExists(capture["table"].to_owned()))
        } else {
          None
        }
      }
      DataFusionError::SchemaError(SchemaError::FieldNotFound {
        field,
        ..
      }) => Some(Self::ColumnDoesntExist(field.name.clone())),
      _ => None,
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
    Self::ParserError(match e {
      ParserError::ParserError(e) => e,
      _ => format!("Error parsing SQL query"),
    })
  }
}

impl From<rocksdb::Error> for Error {
  fn from(e: rocksdb::Error) -> Self {
    tracing::warn!("Rocks db error: {:?}", e);
    Self::IOError(e.into())
  }
}

impl From<DataFusionError> for Error {
  fn from(err: DataFusionError) -> Self {
    Error::from_df_error(&err)
      .unwrap_or_else(|| Self::DataFusionError(Arc::new(err)))
  }
}

impl From<bincode::Error> for Error {
  fn from(e: bincode::Error) -> Self {
    Self::SerdeError(e.into())
  }
}

impl From<Error> for DataFusionError {
  fn from(err: Error) -> Self {
    DataFusionError::External(Box::new(err))
  }
}

impl From<prost::DecodeError> for Error {
  fn from(e: prost::DecodeError) -> Self {
    Self::DecodingError(e.into())
  }
}

impl From<prost::EncodeError> for Error {
  fn from(e: prost::EncodeError) -> Self {
    Self::EncodingError(e.into())
  }
}

impl From<Error> for PgWireError {
  fn from(err: Error) -> Self {
    PgWireError::UserError(
      ErrorInfo::new("ERROR".to_owned(), err.code().to_owned(), err.message())
        .into(),
    )
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

#[cfg(test)]
mod tests {
  use crate::error::RE_TABLE_NOT_FOUND;

  #[test]
  fn table_not_found_regex_test() {
    let caps = RE_TABLE_NOT_FOUND
      .captures("table 'ai_assistant.public.tmp_dqs_nodes2' not found")
      .unwrap();
    assert_eq!(&caps["rel"], "ai_assistant.public.tmp_dqs_nodes2");
  }
}
