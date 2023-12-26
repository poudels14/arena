use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use datafusion::arrow::datatypes::{DataType as DfDataType, Field, TimeUnit};
use postgres_types::Type;
use serde::{Deserialize, Serialize};
use sqlparser::ast::{ColumnDef, DataType as SQLDataType};
use strum_macros::{Display, EnumString};

use crate::{Error, Result};

#[derive(
  Debug, Display, Clone, Serialize, Deserialize, EnumString, PartialEq,
)]
#[repr(u8)]
pub enum DataType {
  // Posgres Type::BOOL
  #[strum(serialize = "BOOL")]
  Boolean = 1,
  // Posgres Type::BYTEA
  #[strum(serialize = "BYTEA")]
  Binary = 2,
  Int16 = 3,
  // Posgres Type::INT4
  #[strum(serialize = "INT4")]
  Int32 = 4,
  // Posgres Type::INT4
  #[strum(serialize = "UINT4")]
  UInt32 = 5,
  // Posgres Type::INT8
  #[strum(serialize = "INT8")]
  Int64 = 6,
  // Posgres Type::INT8
  #[strum(serialize = "UINT8")]
  UInt64 = 7,
  // Posgres Type::VARCHAR
  #[strum(serialize = "VARCHAR")]
  Varchar {
    len: Option<usize>,
  } = 8,
  // Posgres Type::TEXT
  #[strum(serialize = "TEXT")]
  Text = 9,
  // Posgres Type::FLOAT4
  #[strum(serialize = "FLOAT4")]
  Float32 = 10,
  // Posgres Type::FLOAT8
  #[strum(serialize = "FLOAT8")]
  Float64 = 11,
  // Posgres Type::NUMERIC
  #[strum(serialize = "NUMERIC")]
  Decimal {
    p: u8,
    s: i8,
  } = 12,
  // Posgres Type::JSONB
  #[strum(serialize = "JSONB")]
  Jsonb = 13,
  // Posgres Type::JSONB
  #[strum(serialize = "FLOAT4_ARRAY")]
  Vector {
    len: usize,
  } = 14,
  // Posgres Type::TIMESTAMP
  #[strum(serialize = "TIMESTAMP")]
  Timestamp = 15,
}

impl DataType {
  pub fn from_column_def(
    column_def: &ColumnDef,
    df_field: &Field,
  ) -> Result<Self> {
    match &column_def.data_type {
      SQLDataType::Varchar(len) => {
        let len = len.map(|l| l.length as usize);
        Ok(DataType::Varchar { len })
      }
      SQLDataType::Custom(object_name, data) => {
        let data_type_str = object_name.0[0].value.to_uppercase();
        match data_type_str.as_str() {
          "JSONB" => Ok(DataType::Jsonb),
          "VECTOR" => {
            let len = data
              .get(0)
              .and_then(|v| v.parse::<usize>().ok())
              .ok_or_else(|| {
                Error::InvalidDataType(format!(
                  "Size param missing from Vector(size) data type"
                ))
              })?;

            Ok(DataType::Vector { len })
          }
          _ => {
            Err(Error::UnsupportedDataType(column_def.data_type.to_string()))
          }
        }
      }
      _ => DataType::from_field(df_field),
    }
  }

  pub fn from_field(field: &Field) -> Result<Self> {
    match field.data_type() {
      DfDataType::Boolean => Ok(Self::Boolean),
      DfDataType::Int32 => Ok(Self::Int32),
      DfDataType::UInt32 => Ok(Self::UInt32),
      DfDataType::Int64 => Ok(Self::Int64),
      DfDataType::UInt64 => Ok(Self::UInt64),
      DfDataType::Utf8 => {
        let metadata = field.metadata();
        let dt = match Self::from_str(metadata.get("TYPE").unwrap()).unwrap() {
          Self::Timestamp => Self::Timestamp,
          _ => Self::Text,
        };
        Ok(dt)
      }
      DfDataType::Float32 => Ok(Self::Float32),
      DfDataType::Float64 => Ok(Self::Float64),
      DfDataType::Timestamp(TimeUnit::Nanosecond, _) => Ok(Self::Timestamp),
      DfDataType::Decimal256(76, 1) => {
        let metadata = field.metadata();
        let len = metadata.get("LENGTH").and_then(|l| l.parse::<usize>().ok());
        match metadata.get("TYPE").map(|t| t.as_str()) {
          Some("JSON") => Ok(Self::Jsonb),
          Some("VECTOR") => Ok(Self::Vector { len: len.unwrap() }),
          v => panic!("Invalid \"TYPE\" in field metadata: {:?}", v),
        }
      }
      DfDataType::Binary => Ok(Self::Binary),
      v => {
        if let DfDataType::List(sub_field) = v {
          match *sub_field.data_type() {
            DfDataType::Float32 => {
              if let Some(len) = field.metadata().get("LENGTH") {
                let len = len.parse::<usize>().unwrap();
                return Ok(Self::Vector { len });
              }
            }
            _ => {}
          }
        }
        Err(Error::UnsupportedDataType(format!(
          "Data type {:?} not supported",
          v
        )))
      }
    }
  }

  pub fn to_oid(&self) -> u32 {
    match self {
      Self::Boolean => Type::BOOL.oid(),
      Self::Binary => Type::BYTEA.oid(),
      Self::Int16 => Type::INT2.oid(),
      Self::Int32 => Type::INT4.oid(),
      Self::UInt32 => Type::INT8.oid(),
      Self::Int64 => Type::INT8.oid(),
      Self::UInt64 => Type::INT8.oid(),
      Self::Varchar { .. } => Type::VARCHAR.oid(),
      Self::Text => Type::TEXT.oid(),
      Self::Float32 => Type::FLOAT4.oid(),
      Self::Float64 => Type::FLOAT8.oid(),
      Self::Decimal { p: _, s: _ } => Type::NUMERIC.oid(),
      Self::Jsonb => Type::JSONB.oid(),
      Self::Vector { .. } => Type::FLOAT4_ARRAY.oid(),
      Self::Timestamp => Type::TIMESTAMP.oid(),
    }
  }

  /// Returns the Datafusion datatype corresponding to arena datatype
  /// as well as additional metadata for that datatype
  pub fn to_df_datatype(&self) -> (DfDataType, HashMap<String, String>) {
    let mut metadata = HashMap::from([("TYPE".to_owned(), self.to_string())]);
    let df_data_type = match self {
      Self::Boolean => DfDataType::Boolean,
      Self::Int16 => DfDataType::Int16,
      Self::Int32 => DfDataType::Int32,
      Self::UInt32 => DfDataType::UInt32,
      Self::Int64 => DfDataType::Int64,
      Self::UInt64 => DfDataType::UInt64,
      Self::Float32 => DfDataType::Float32,
      Self::Float64 => DfDataType::Float64,
      Self::Decimal { p, s } => DfDataType::Decimal128(*p, *s),
      Self::Binary => DfDataType::Binary,
      Self::Text => DfDataType::Utf8,
      Self::Varchar { len } => {
        if let Some(len) = len {
          metadata.insert("LENGTH".to_owned(), len.to_string());
        }
        DfDataType::Utf8
      }
      Self::Jsonb => DfDataType::Utf8,
      Self::Vector { len } => {
        metadata.insert("LENGTH".to_owned(), len.to_string());
        DfDataType::List(Arc::new(Field::new(
          "item",
          DfDataType::Float32,
          true,
        )))
      }
      Self::Timestamp => DfDataType::Utf8,
    };

    (df_data_type, metadata)
  }
}
