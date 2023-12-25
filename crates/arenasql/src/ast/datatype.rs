use sqlparser::ast::{
  DataType as SQLDataType, ExactNumberInfo, Statement as SQLStatement,
};

use crate::Result;

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
          // Postgres JSONB, VECTOR, etc will be parsed as Custom data type
          SQLDataType::Custom(_, _) => {
            col.data_type =
              SQLDataType::Decimal(ExactNumberInfo::PrecisionAndScale(76, 1));
          }
          _ => {}
        }
      }
      Ok(())
    }
    _ => Ok(()),
  }
}
