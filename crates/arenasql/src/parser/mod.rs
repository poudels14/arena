use sqlparser::ast::{
  DataType as SQLDataType, ExactNumberInfo, ObjectName,
  Statement as SQLStatement,
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
          // Note: dont support Decimal with precision >= 40 since
          // precision >= 40 is reserved for encoding data type that
          // datafusion doesn't support like JSONB, VECTOR(size), etc.
          SQLDataType::Decimal(ExactNumberInfo::PrecisionAndScale(p, _)) => {
            if *p >= 40 {
              bail!(Error::UnsupportedDataType(format!(
                "Only decimal with precision < 40",
              )));
            }
          }
          // Postgres JSONB will be parsed as Custom data type
          SQLDataType::Custom(object, data) => {
            if let Some(data_type) =
              convert_to_decimal_data_type(&object, &data)?
            {
              col.data_type = data_type;
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

/// TODO: THIS IS SUPER HACKY!!!
/// This converts custom data type that we want to support to something
/// that DataFusion supports in order to prevent errors thrown by type
/// checking in DataFusion
fn convert_to_decimal_data_type(
  object: &ObjectName,
  data: &Vec<String>,
) -> Result<Option<SQLDataType>> {
  let data_type_str = object.0[0].value.to_uppercase();
  match data_type_str.as_str() {
    "JSONB" => Ok(Some(SQLDataType::Decimal(
      ExactNumberInfo::PrecisionAndScale(41, 1),
    ))),
    // Precision 50 and beyond is used by VECTOR to encode
    // vector length
    "VECTOR" => {
      let len =
        data
          .get(0)
          .and_then(|v| v.parse::<u64>().ok())
          .ok_or_else(|| {
            Error::InvalidDataType(format!(
              "Size param missing from Vector(size) data type"
            ))
          })?;

      if len % 4 != 0 {
        bail!(Error::UnsupportedDataType(format!(
          "Vector length must be multiple of 4 but is {}",
          len
        )));
      } else if len < 4 || len > 5200 {
        bail!(Error::UnsupportedDataType(format!(
          "Vector length must be <= 5200 but is {}",
          len
        )));
      }

      let len_by_4 = len / 4;
      Ok(Some(SQLDataType::Decimal(
        ExactNumberInfo::PrecisionAndScale(
          // precision is between [50 and 76) and scale is between [0 and 50]
          // this is needed to not violate the scale <= precision constraint
          (len_by_4 % 26) + 50,
          len_by_4 / 26,
        ),
      )))
    }
    _ => Ok(None),
  }
}

#[cfg(test)]
mod tests {
  use sqlparser::ast::{
    DataType as SQLDataType, ExactNumberInfo, Ident, ObjectName,
  };

  use super::convert_to_decimal_data_type;

  fn vector() -> ObjectName {
    ObjectName(vec![Ident::new("VECTOR")])
  }

  #[test]
  fn test_datatype_conversion_for_vector() {
    vec![
      ("4", 51, 0),
      ("8", 52, 0),
      ("12", 53, 0),
      ("104", 50, 1),
      ("108", 51, 1),
      ("112", 52, 1),
      ("208", 50, 2),
      ("212", 51, 2),
      ("768", 60, 7),
      ("5192", 74, 49),
      ("5196", 75, 49),
      ("5200", 50, 50),
    ]
    .into_iter()
    .for_each(|(len, precision, scale)| {
      let Some(encoded_decimal) =
        convert_to_decimal_data_type(&vector(), &vec![len.to_owned()]).unwrap()
      else {
        panic!("Expected valid encoded Decimal SQL datatype")
      };

      assert_eq!(
        encoded_decimal,
        SQLDataType::Decimal(ExactNumberInfo::PrecisionAndScale(
          precision, scale
        )),
        "Failed converting Vector({}) to List(Decimal)",
        len
      );
    })
  }
}
