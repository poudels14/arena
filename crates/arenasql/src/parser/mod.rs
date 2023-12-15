mod datatype;

pub use datatype::cast_unsupported_data_types;

use sqlparser::ast::Statement as SQLStatement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

use crate::Result;

pub fn parse(sql: &str) -> Result<Vec<SQLStatement>> {
  let dialect = PostgreSqlDialect {};
  Ok(Parser::parse_sql(&dialect, sql)?)
}

/// Parses the query and "sanitizes" the statements so that they
/// can run in Datafusion.
/// The sanitizer updates the statements to support features like
/// JSONB datatype etc
pub fn parse_and_sanitize(sql: &str) -> Result<Vec<SQLStatement>> {
  let mut statements = parse(sql)?;
  statements
    .iter_mut()
    .map(|stmt| cast_unsupported_data_types(stmt))
    .collect::<Result<()>>()?;
  Ok(statements)
}
