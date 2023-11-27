use datafusion::arrow::array::{
  ArrayRef, BooleanArray, Float32Array, Float64Array, Int32Array, Int64Array,
  StringArray,
};
use serde::{Deserialize, Serialize};

use super::DataType;

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
  pub fn array_ref_to_vec<'a>(
    data_type: &DataType,
    data: &'a ArrayRef,
  ) -> Vec<SerializedCell<&'a [u8]>> {
    match data_type {
      DataType::Null => (0..data.len())
        .into_iter()
        .map(|_| SerializedCell::Null)
        .collect(),

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
      DataType::Varchar { len: _ } | DataType::Text => data
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap()
        .iter()
        .map(|v| {
          v.map(|v| SerializedCell::Blob(v.as_bytes()))
            .unwrap_or_default()
        })
        .collect(),
      _ => unimplemented!(),
    }
  }

  pub fn vec_to_array_ref<'a>(
    _data_type: &DataType,
    _data: &'a ArrayRef,
  ) -> Vec<SerializedCell<&'a [u8]>> {
    unimplemented!()
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
