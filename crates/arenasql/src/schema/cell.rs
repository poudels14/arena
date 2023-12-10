use datafusion::arrow::array::{
  ArrayRef, BooleanArray, Float32Array, Float64Array, Int32Array, Int64Array,
  StringArray,
};
use datafusion::error::Result;
use datafusion::scalar::ScalarValue;
use serde::{Deserialize, Serialize};

use super::{Column, DataType};
use crate::error::null_constraint_violation;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum SerializedCell<T> {
  Null = 1,
  Boolean(bool) = 2,
  Int32(i32) = 3,
  Int64(i64) = 4,
  Float32(f32) = 5,
  Float64(f64) = 6,
  Blob(T) = 7,
}

impl Default for SerializedCell<Vec<u8>> {
  fn default() -> Self {
    SerializedCell::Null
  }
}
impl Default for SerializedCell<&[u8]> {
  fn default() -> Self {
    SerializedCell::Null
  }
}

#[macro_export]
macro_rules! data_with_value {
  ($data:ident, $arr_type:ident, $mapper:expr) => {
    $data
      .as_any()
      .downcast_ref::<$arr_type>()
      .unwrap()
      .iter()
      .map($mapper)
      .collect()
  };
}

impl SerializedCell<Vec<u8>> {
  pub fn from_scalar(scalar: &ScalarValue) -> Self {
    match scalar {
      ScalarValue::Null => Self::Null,
      ScalarValue::Boolean(v) => {
        v.map(|v| Self::Boolean(v)).unwrap_or_default()
      }
      ScalarValue::Int32(v) => v.map(|v| Self::Int32(v)).unwrap_or_default(),
      ScalarValue::Int64(v) => v.map(|v| Self::Int64(v)).unwrap_or_default(),
      ScalarValue::Float32(v) => {
        v.map(|v| Self::Float32(v)).unwrap_or_default()
      }
      ScalarValue::Float64(v) => {
        v.map(|v| Self::Float64(v)).unwrap_or_default()
      }
      ScalarValue::Utf8(v) | ScalarValue::LargeUtf8(v) => v
        .as_ref()
        .map(|v| Self::Blob(v.as_bytes().to_vec()))
        .unwrap_or_default(),
      _ => unimplemented!(),
    }
  }

  pub fn array_ref_to_vec<'a>(
    table_name: &str,
    column: &Column,
    data: &'a ArrayRef,
  ) -> Result<Vec<SerializedCell<&'a [u8]>>> {
    if !column.nullable && data.null_count() > 0 {
      return Err(null_constraint_violation(table_name, &column.name));
    }

    Ok(match &column.data_type {
      DataType::Boolean => {
        data_with_value!(data, BooleanArray, |v| v
          .map(|v| SerializedCell::Boolean(v))
          .unwrap_or_default())
      }
      DataType::Int32 => {
        data_with_value!(data, Int32Array, |v| v
          .map(|v| SerializedCell::Int32(v))
          .unwrap_or_default())
      }
      DataType::Int64 => {
        data_with_value!(data, Int64Array, |v| v
          .map(|v| SerializedCell::Int64(v))
          .unwrap_or_default())
      }
      DataType::Float32 => {
        data_with_value!(data, Float32Array, |v| v
          .map(|v| SerializedCell::Float32(v))
          .unwrap_or_default())
      }
      DataType::Float64 => {
        data_with_value!(data, Float64Array, |v| v
          .map(|v| SerializedCell::Float64(v))
          .unwrap_or_default())
      }
      DataType::Varchar { len: _ } | DataType::Text => {
        data_with_value!(data, StringArray, |v| {
          v.map(|v| SerializedCell::Blob(v.as_bytes()))
            .unwrap_or_default()
        })
      }
      _ => unimplemented!(),
    })
  }

  #[inline]
  pub fn as_bool(self) -> Option<bool> {
    match self {
      SerializedCell::Null => None,
      SerializedCell::Boolean(value) => Some(value),
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn as_i32(self) -> Option<i32> {
    match self {
      SerializedCell::Null => None,
      SerializedCell::Int32(value) => Some(value),
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn as_i64(self) -> Option<i64> {
    match self {
      SerializedCell::Null => None,
      SerializedCell::Int64(value) => Some(value),
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn as_f32(self) -> Option<f32> {
    match self {
      SerializedCell::Null => None,
      SerializedCell::Float32(value) => Some(value),
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn as_f64(self) -> Option<f64> {
    match self {
      SerializedCell::Null => None,
      SerializedCell::Float64(value) => Some(value),
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn as_bytes(self) -> Option<Vec<u8>> {
    match self {
      SerializedCell::Null => None,
      SerializedCell::Blob(value) => Some(value),
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn as_string(self) -> Option<String> {
    match self {
      SerializedCell::Null => None,
      SerializedCell::Blob(bytes) => unsafe {
        Some(String::from_utf8_unchecked(bytes))
      },
      v => unreachable!("Trying to convert {:?} to string", &v),
    }
  }
}

impl<'a> SerializedCell<&'a [u8]> {
  #[inline]
  pub fn as_bool(&self) -> Option<bool> {
    match self {
      SerializedCell::Null => None,
      SerializedCell::Boolean(value) => Some(*value),
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn as_i32(&self) -> Option<i32> {
    match self {
      SerializedCell::Null => None,
      SerializedCell::Int32(value) => Some(*value),
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn as_i64(&self) -> Option<i64> {
    match self {
      SerializedCell::Null => None,
      SerializedCell::Int64(value) => Some(*value),
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn as_f32(&self) -> Option<f32> {
    match self {
      SerializedCell::Null => None,
      SerializedCell::Float32(value) => Some(*value),
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn as_f64(&self) -> Option<f64> {
    match self {
      SerializedCell::Null => None,
      SerializedCell::Float64(value) => Some(*value),
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn as_bytes(&self) -> Option<&'a [u8]> {
    match self {
      SerializedCell::Null => None,
      SerializedCell::Blob(value) => Some(value),
      _ => unreachable!(),
    }
  }

  #[inline]
  pub fn as_str(&self) -> Option<&'a str> {
    match self {
      SerializedCell::Null => None,
      SerializedCell::Blob(bytes) => unsafe {
        Some(std::str::from_utf8_unchecked(bytes))
      },
      v => unreachable!("Trying to convert {:?} to string", &v),
    }
  }
}

impl<'a> From<&SerializedCell<&'a [u8]>> for SerializedCell<Vec<u8>> {
  fn from(cell: &SerializedCell<&'a [u8]>) -> Self {
    match *cell {
      SerializedCell::Null => Self::Null,
      SerializedCell::Boolean(v) => Self::Boolean(v),
      SerializedCell::Int32(v) => Self::Int32(v),
      SerializedCell::Int64(v) => Self::Int64(v),
      SerializedCell::Float32(v) => Self::Float32(v),
      SerializedCell::Float64(v) => Self::Float64(v),
      SerializedCell::Blob(blob) => Self::Blob(blob.to_vec()),
    }
  }
}
