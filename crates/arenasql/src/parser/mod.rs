use sqlparser::ast::Statement as SQLStatement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

use crate::error::Error;

pub fn parse(sql: &str) -> Result<Vec<SQLStatement>, Error> {
  let dialect = PostgreSqlDialect {};
  Ok(Parser::parse_sql(&dialect, sql)?)
}
