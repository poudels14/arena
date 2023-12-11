use sqlparser::ast::{
  DataType as SQLDataType, ExactNumberInfo, Ident, Statement as SQLStatement,
};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

use crate::error::Error;
use crate::{bail, Result};

pub fn parse(sql: &str) -> Result<Vec<SQLStatement>> {
  let dialect = PostgreSqlDialect {};
  Ok(Parser::parse_sql(&dialect, sql)?)
}

/// This changes the datatypes of the columns in the `CREATE TABLE` query
/// to something that Datafusion supports but internally means different
/// data type. This is done because datafusion doesn't support types like
/// JSON, JSONB, vector but we need to support them
pub fn cast_unsupported_data_types(stmt: &mut SQLStatement) -> Result<()> {
  match stmt {
    SQLStatement::CreateTable {
      ref mut columns, ..
    } => {
      for col in columns {
        match &col.data_type {
          // Note: dont support Decimal with 76 precision since it's used to
          // encode data type that datafusion doesn't support
          SQLDataType::Decimal(ExactNumberInfo::PrecisionAndScale(76, _)) => {
            bail!(Error::UnsupportedDataType(col.data_type.to_string()));
          }
          // Postgres JSONB will be parsed as Custom data type
          SQLDataType::Custom(object, _) => {
            if object.0[0] == Ident::new("JSONB") {
              col.data_type =
                SQLDataType::Decimal(ExactNumberInfo::PrecisionAndScale(76, 1));
            }
          }
          _ => {}
        }
      }
      Ok(())
    }
    _ => Ok(()),
  }
}
