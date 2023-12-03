use datafusion::arrow::datatypes::DataType as DfDataType;
use postgres_types::Type;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

use crate::Error;

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
  Varchar { len: u32 } = 6,
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
}

impl DataType {
  pub fn to_oid(&self) -> u32 {
    match self {
      Self::Boolean => Type::BOOL.oid(),
      Self::Binary => Type::BYTEA.oid(),
      Self::Int64 => Type::INT8.oid(),
      Self::UInt64 => Type::INT8.oid(),
      Self::Int32 => Type::INT4.oid(),
      Self::Varchar { len: _ } => Type::VARCHAR.oid(),
      Self::Text => Type::TEXT.oid(),
      Self::Float32 => Type::FLOAT4.oid(),
      Self::Float64 => Type::FLOAT8.oid(),
      Self::Decimal { p: _, s: _ } => Type::NUMERIC.oid(),
      Self::Jsonb => Type::JSONB.oid(),
    }
  }
}

impl TryFrom<&DfDataType> for DataType {
  type Error = Error;
  fn try_from(value: &DfDataType) -> Result<Self, Self::Error> {
    match value {
      DfDataType::Boolean => Ok(Self::Boolean),
      DfDataType::Int32 => Ok(Self::Int32),
      DfDataType::Int64 => Ok(Self::Int64),
      DfDataType::UInt64 => Ok(Self::UInt64),
      DfDataType::Utf8 => Ok(Self::Text),
      DfDataType::Float32 => Ok(Self::Float32),
      DfDataType::Float64 => Ok(Self::Float64),
      DfDataType::Decimal128(p, s) => Ok(Self::Decimal { p: *p, s: *s }),
      // Note: We use Decimal256 to represent data types not supported
      // in datafusion like JSONB and don't actually support Decimal128
      DfDataType::Decimal256(p, s) => match (p, s) {
        (76, 1) => Ok(Self::Jsonb),
        _ => Err(Error::UnsupportedDataType(format!(
          "Data type [Decimal256({}, {})] not supported",
          p, s
        ))),
      },
      DfDataType::Binary => Ok(Self::Binary),
      v => Err(Error::UnsupportedDataType(format!(
        "Data type [{}] not supported",
        v
      ))),
    }
  }
}

impl Into<DfDataType> for DataType {
  fn into(self) -> DfDataType {
    match self {
      Self::Boolean => DfDataType::Boolean,
      Self::Int32 => DfDataType::Int32,
      Self::Int64 => DfDataType::Int64,
      Self::UInt64 => DfDataType::UInt64,
      Self::Text => DfDataType::Utf8,
      Self::Float32 => DfDataType::Float32,
      Self::Float64 => DfDataType::Float64,
      Self::Decimal { p, s } => DfDataType::Decimal128(p, s),
      Self::Binary => DfDataType::Binary,
      Self::Jsonb => DfDataType::Utf8,
      Self::Varchar { .. } => DfDataType::Utf8,
    }
  }
}
