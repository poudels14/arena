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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum DataWithValue<T> {
  Null,
  Boolean(bool),
  Int32(i32),
  Int64(i64),
  Float32(f32),
  Float64(f64),
  Blob(T),
}

impl Default for DataWithValue<Vec<u8>> {
  fn default() -> Self {
    DataWithValue::Null
  }
}
impl Default for DataWithValue<&[u8]> {
  fn default() -> Self {
    DataWithValue::Null
  }
}

impl DataWithValue<Vec<u8>> {
  #[inline]
  pub fn as_bool(self) -> Option<bool> {
    match self {
      DataWithValue::Null => None,
      DataWithValue::Boolean(value) => Some(value),
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn as_i32(self) -> Option<i32> {
    match self {
      DataWithValue::Null => None,
      DataWithValue::Int32(value) => Some(value),
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn as_i64(self) -> Option<i64> {
    match self {
      DataWithValue::Null => None,
      DataWithValue::Int64(value) => Some(value),
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn as_f32(self) -> Option<f32> {
    match self {
      DataWithValue::Null => None,
      DataWithValue::Float32(value) => Some(value),
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn as_f64(self) -> Option<f64> {
    match self {
      DataWithValue::Null => None,
      DataWithValue::Float64(value) => Some(value),
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn as_bytes(self) -> Option<Vec<u8>> {
    match self {
      DataWithValue::Null => None,
      DataWithValue::Blob(value) => Some(value),
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn as_string(self) -> Option<String> {
    match self {
      DataWithValue::Null => None,
      DataWithValue::Blob(bytes) => unsafe {
        Some(String::from_utf8_unchecked(bytes))
      },
      v => unreachable!("Trying to convert {:?} to string", &v),
    }
  }
}
