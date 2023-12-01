use datafusion::arrow::datatypes::DataType as DfDataType;
use postgres_types::Type;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

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

impl From<&DfDataType> for DataType {
  fn from(value: &DfDataType) -> Self {
    match value {
      DfDataType::Boolean => Self::Boolean,
      DfDataType::Int32 => Self::Int32,
      DfDataType::Int64 => Self::Int64,
      DfDataType::UInt64 => Self::UInt64,
      DfDataType::Utf8 => Self::Text,
      DfDataType::Float32 => Self::Float32,
      DfDataType::Float64 => Self::Float64,
      DfDataType::Decimal128(p, s) => Self::Decimal { p: *p, s: *s },
      v => unimplemented!("Data type [{}] not supported", v),
    }
  }
}

impl Into<DfDataType> for DataType {
  fn into(self) -> DfDataType {
    match self {
      Self::Boolean => DfDataType::Boolean,
      Self::Int32 => DfDataType::Int32,
      Self::Int64 => DfDataType::Int64,
      Self::Text => DfDataType::Utf8,
      Self::Float32 => DfDataType::Float32,
      Self::Float64 => DfDataType::Float64,
      Self::Decimal { p, s } => DfDataType::Decimal128(p, s),
      v => unimplemented!("Data type [{:?}] not supported", v),
    }
  }
}
