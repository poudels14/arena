use std::sync::Arc;

use datafusion::arrow::array::{
  ArrayRef, BooleanArray, Float32Array, Float64Array, Int32Array, Int64Array,
  ListArray, StringArray,
};
use datafusion::error::Result;
use datafusion::scalar::ScalarValue;
use serde::{Deserialize, Serialize};

use super::{Column, DataType};
use crate::error::null_constraint_violation;
use crate::{bail, df_error, Error};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum SerializedCell<'a> {
  Null = 1,
  Boolean(bool) = 2,
  Int32(i32) = 3,
  Int64(i64) = 4,
  Float32(f32) = 5,
  Float64(f64) = 6,
  // Using the reference for bytes prevents data cloning during
  // deserialization
  Blob(&'a [u8]) = 7,
  // TODO: convert f32 to u16 when storing in order to store bfloat16
  // Vec<f32> can't be deserialized to &'a [f32] because converting [u8]
  // to f32 requires allocation
  Vector(Arc<Vec<f32>>) = 8,
}

// Note: this should only be used when it's impossible to use
// SerializeCell<'a>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum OwnedSerializedCell {
  Null = 1,
  Boolean(bool) = 2,
  Int32(i32) = 3,
  Int64(i64) = 4,
  Float32(f32) = 5,
  Float64(f64) = 6,
  Blob(Arc<Vec<u8>>) = 7,
  Vector(Arc<Vec<f32>>) = 8,
}

impl<'a> Default for SerializedCell<'a> {
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

impl<'a> SerializedCell<'a> {
  pub fn from_scalar(scalar: &'a ScalarValue) -> Self {
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
        .map(|v| Self::Blob(v.as_bytes()))
        .unwrap_or_default(),
      _ => unimplemented!(),
    }
  }
}

impl<'a> SerializedCell<'a> {
  pub fn array_ref_to_vec<'b>(
    table_name: &str,
    column: &Column,
    data: &'b ArrayRef,
  ) -> Result<Vec<SerializedCell<'b>>> {
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
      DataType::Jsonb => {
        data_with_value!(data, StringArray, |v| {
          v.map(|v| SerializedCell::Blob(v.as_bytes()))
            .unwrap_or_default()
        })
      }
      DataType::Vector { len } => {
        let vec: Result<Vec<SerializedCell<'b>>> = data_with_value!(
          data,
          ListArray,
          |v| {
            v.map(|v| {
              let vector = Arc::new(
                v.as_any()
                  .downcast_ref::<Float32Array>()
                  .unwrap()
                  .iter()
                  .map(|f| f.unwrap())
                  .collect::<Vec<f32>>(),
              );
              if vector.len() != *len {
                bail!(df_error!(Error::InvalidQuery(format!(
                  "Expected vector of length \"{}\" but got vector of length \"{}\""
                , len, vector.len()))));
              }
              Ok(SerializedCell::Vector::<'b>(vector))
            })
            .unwrap()
          }
        );
        vec?
      }
      _ => unimplemented!(),
    })
  }

  // Note: this clones the data, so use it as little as possible
  // This is meant to be used mostly during error generation
  pub fn to_owned(&self) -> OwnedSerializedCell {
    match *self {
      Self::Null => OwnedSerializedCell::Null,
      Self::Boolean(v) => OwnedSerializedCell::Boolean(v),
      Self::Int32(v) => OwnedSerializedCell::Int32(v),
      Self::Int64(v) => OwnedSerializedCell::Int64(v),
      Self::Float32(v) => OwnedSerializedCell::Float32(v),
      Self::Float64(v) => OwnedSerializedCell::Float64(v),
      Self::Blob(blob) => OwnedSerializedCell::Blob(Arc::new(blob.to_vec())),
      Self::Vector(ref v) => OwnedSerializedCell::Vector(Arc::new(v.to_vec())),
    }
  }

  #[inline]
  pub fn is_null(&self) -> bool {
    match self {
      Self::Null => true,
      _ => false,
    }
  }

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

  #[inline]
  pub fn as_vector(&self) -> Option<Arc<Vec<f32>>> {
    match self {
      SerializedCell::Null => None,
      SerializedCell::Vector(v) => Some(v.clone()),
      v => unreachable!("Trying to convert {:?} to float vector", &v),
    }
  }
}
