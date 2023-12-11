use std::collections::HashMap;
use std::sync::Arc;

use datafusion::arrow::datatypes::{DataType as DfDataType, Field};
use postgres_types::Type;
use serde::{Deserialize, Serialize};
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
  // Posgres Type::INT8
  #[strum(serialize = "INT8")]
  Int64 = 3,
  // Posgres Type::INT4
  #[strum(serialize = "UINT8")]
  UInt64 = 4,
  // Posgres Type::INT4
  #[strum(serialize = "INT4")]
  Int32 = 5,
  // Posgres Type::VARCHAR
  #[strum(serialize = "VARCHAR")]
  Varchar { len: usize } = 6,
  // Posgres Type::TEXT
  #[strum(serialize = "TEXT")]
  Text = 7,
  // Posgres Type::FLOAT4
  #[strum(serialize = "FLOAT4")]
  Float32 = 8,
  // Posgres Type::FLOAT8
  #[strum(serialize = "FLOAT8")]
  Float64 = 9,
  // Posgres Type::NUMERIC
  #[strum(serialize = "NUMERIC")]
  Decimal { p: u8, s: i8 } = 10,
  // Posgres Type::JSONB
  #[strum(serialize = "JSONB")]
  Jsonb = 11,
  // Posgres Type::JSONB
  #[strum(serialize = "FLOAT4_ARRAY")]
  Vector { len: usize } = 12,
}

impl DataType {
  pub fn from_field(field: &Field) -> Result<Self> {
    match field.data_type() {
      DfDataType::Boolean => Ok(Self::Boolean),
      DfDataType::Int32 => Ok(Self::Int32),
      DfDataType::Int64 => Ok(Self::Int64),
      DfDataType::UInt64 => Ok(Self::UInt64),
      DfDataType::Utf8 => Ok(Self::Text),
      DfDataType::Float32 => Ok(Self::Float32),
      DfDataType::Float64 => Ok(Self::Float64),
      // Note: We use Decimal256 to represent data types not supported
      // in datafusion like JSONB and don't actually support Decimal128
      DfDataType::Decimal256(p, s) if *p == 41_u8 && *s == 1_i8 => {
        Ok(Self::Jsonb)
      }
      DfDataType::Decimal256(p, s) if *p > 50 => Ok(Self::Vector {
        len: 4 * (*s as usize * 50 + (*p - 50) as usize),
      }),
      DfDataType::Decimal256(p, s) => Err(Error::UnsupportedDataType(format!(
        "Data type [Decimal256({}, {})] not supported",
        p, s
      ))),
      DfDataType::Binary => Ok(Self::Binary),
      v => {
        if let DfDataType::List(sub_field) = v {
          match *sub_field.data_type() {
            DfDataType::Float32 => {
              if let Some(len) = field.metadata().get("len") {
                let len = len.parse::<usize>().unwrap();
                return Ok(Self::Vector { len });
              }
            }
            _ => {}
          }
        }
        Err(Error::UnsupportedDataType(format!(
          "Data type [{}] not supported",
          v
        )))
      }
    }
  }

  pub fn to_oid(&self) -> u32 {
    match self {
      Self::Boolean => Type::BOOL.oid(),
      Self::Binary => Type::BYTEA.oid(),
      Self::Int64 => Type::INT8.oid(),
      Self::UInt64 => Type::INT8.oid(),
      Self::Int32 => Type::INT4.oid(),
      Self::Varchar { .. } => Type::VARCHAR.oid(),
      Self::Text => Type::TEXT.oid(),
      Self::Float32 => Type::FLOAT4.oid(),
      Self::Float64 => Type::FLOAT8.oid(),
      Self::Decimal { p: _, s: _ } => Type::NUMERIC.oid(),
      Self::Jsonb => Type::JSONB.oid(),
      Self::Vector { .. } => Type::FLOAT4_ARRAY.oid(),
    }
  }

  /// Returns the Datafusion datatype corresponding to arena datatype
  /// as well as additional metadata for that datatype
  pub fn to_df_datatype(&self) -> (DfDataType, HashMap<String, String>) {
    match self {
      Self::Boolean => (DfDataType::Boolean, HashMap::new()),
      Self::Int32 => (DfDataType::Int32, HashMap::new()),
      Self::Int64 => (DfDataType::Int64, HashMap::new()),
      Self::UInt64 => (DfDataType::UInt64, HashMap::new()),
      Self::Float32 => (DfDataType::Float32, HashMap::new()),
      Self::Float64 => (DfDataType::Float64, HashMap::new()),
      Self::Decimal { p, s } => {
        (DfDataType::Decimal128(*p, *s), HashMap::new())
      }
      Self::Binary => (DfDataType::Binary, HashMap::new()),
      Self::Text | Self::Varchar { .. } | Self::Jsonb => {
        (DfDataType::Utf8, HashMap::new())
      }
      Self::Vector { len } => (
        DfDataType::List(Arc::new(Field::new(
          "item",
          DfDataType::Float32,
          true,
        ))),
        HashMap::from([("len".to_owned(), len.to_string())]),
      ),
    }
  }
}
