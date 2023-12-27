use std::fmt;
use std::sync::Arc;

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
  SerdeError(String),
  InsufficientPrivilege,
  DataFusionError(Arc<DataFusionError>),
  ReservedWord(String),
}

const RE_TABLE_NOT_FOUND: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"table '(?P<rel>[\w.]+)' not found").unwrap());

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
      // undefined_table or index
      Self::RelationDoesntExist(_) => "42P01",
      // duplicate_table or duplicate index
      Self::RelationAlreadyExists(_) => "42P07",
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
      | Self::ColumnDoesntExist(_)
      | Self::IOError(_)
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
      | Self::IOError(msg)
      | Self::SerdeError(msg)
      | Self::ReservedWord(msg)
      | Self::InvalidParameter(msg)
      | Self::InvalidTransactionState(msg) => msg.to_owned(),
      Self::InsufficientPrivilege => format!("permission denied"),
      Self::InternalError(msg) => {
        eprintln!("Internal error: {:?}", msg);
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
      Self::DataFusionError(msg) => match msg.as_ref() {
        DataFusionError::External(err) => {
          if let Some(arena_err) = err.downcast_ref::<Error>() {
            arena_err.message()
          } else {
            eprintln!("Error: {:?}", err);
            format!("Internal error")
          }
        }
        DataFusionError::Context(_, e) => match e.as_ref() {
          DataFusionError::External(err) => {
            if let Some(arena_err) = err.downcast_ref::<Error>() {
              arena_err.message()
            } else {
              eprintln!("Unknown error at {}:{}: {:?}", file!(), line!(), err);
              format!("Internal error")
            }
          }
          err => {
            eprintln!("Unknown error at {}:{}: {:?}", file!(), line!(), err);
            format!("Internal error")
          }
        },
        DataFusionError::Plan(msg) => {
          if msg.contains("not yet supported") || msg.contains("not supported")
          {
            eprintln!(
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
            eprintln!(
              "Unknown query error: {}:{}: {:?}",
              file!(),
              line!(),
              msg
            );
            format!("Unknown error")
          }
        }
        DataFusionError::Execution(msg) => {
          let lowercase_msg = msg.to_lowercase();
          if lowercase_msg.contains("table")
            && lowercase_msg.contains("doesn't exist")
          {
            lowercase_msg
          } else {
            eprintln!("Execution error at {}:{}: {:?}", file!(), line!(), msg);
            format!("Internal error")
          }
        }
        err => {
          eprintln!("Unknown error at {}:{}: {:?}", file!(), line!(), err);
          format!("Internal error")
        }
      },
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
    Self::IOError(e.into_string())
  }
}

impl From<DataFusionError> for Error {
  fn from(err: DataFusionError) -> Self {
    match err {
      DataFusionError::Plan(ref msg) => {
        if let Some(capture) = RE_TABLE_NOT_FOUND.captures(msg) {
          return Self::RelationDoesntExist(capture["rel"].to_owned());
        }
      }
      _ => {}
    }

    Self::DataFusionError(Arc::new(err))
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
