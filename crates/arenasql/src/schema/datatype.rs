use datafusion::arrow::datatypes::DataType as DfDataType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum DataType {
  Null = 1,
  Boolean = 2,
  Int32 = 3,
  Int64 = 4,
  Varchar { len: u32 } = 5,
  Text = 6,
  Jsonb = 7,
  Decimal { p: u8, s: i8 } = 8,
  Float32 = 9,
  Float64 = 10,
  Binary = 11,
}

impl TryFrom<&DfDataType> for DataType {
  type Error = crate::error::Error;
  fn try_from(value: &DfDataType) -> Result<Self, Self::Error> {
    match value {
      DfDataType::Null => Ok(Self::Null),
      DfDataType::Boolean => Ok(Self::Boolean),
      DfDataType::Int32 => Ok(Self::Int32),
      DfDataType::Int64 => Ok(Self::Int64),
      DfDataType::Utf8 => Ok(Self::Text),
      DfDataType::Float32 => Ok(Self::Float32),
      DfDataType::Float64 => Ok(Self::Float64),
      DfDataType::Decimal128(p, s) => Ok(Self::Decimal { p: *p, s: *s }),
      v => unimplemented!("Data type [{}] not supported", v),
    }
  }
}

impl Into<DfDataType> for DataType {
  fn into(self) -> DfDataType {
    match self {
      Self::Null => DfDataType::Null,
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
