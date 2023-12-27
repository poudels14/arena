use std::sync::Arc;

use chrono::{NaiveDateTime, SecondsFormat};
use datafusion::arrow::array::{
  as_boolean_array, as_generic_list_array, as_primitive_array, as_string_array,
  ArrayRef,
};
use datafusion::arrow::datatypes::{
  Float32Type, Float64Type, Int16Type, Int32Type, Int64Type, UInt32Type,
  UInt64Type,
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
  Int16(i16) = 3,
  // Use i16 for u16 since pgwire doesn't support u16
  // Same for u64
  Int32(i32) = 4,
  UInt32(u32) = 5,
  Int64(i64) = 6,
  UInt64(u64) = 7,
  Float32(f32) = 8,
  Float64(f64) = 9,
  // Using the reference for bytes prevents data cloning during
  // deserialization
  Blob(&'a [u8]) = 10,
  // TODO: convert f32 to u16 when storing in order to store bfloat16
  // Vec<f32> can't be deserialized to &'a [f32] because converting [u8]
  // to f32 requires allocation
  Vector(Arc<Vec<f32>>) = 11,
  Timestamp(i64) = 12,
}

// Note: this should only be used when it's impossible to use
// SerializeCell<'a>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum OwnedSerializedCell {
  Null = 1,
  Boolean(bool) = 2,
  Int16(i16) = 3,
  Int32(i32) = 4,
  UInt32(u32) = 5,
  Int64(i64) = 6,
  UInt64(u64) = 7,
  Float32(f32) = 8,
  Float64(f64) = 9,
  Blob(Arc<Vec<u8>>) = 10,
  Vector(Arc<Vec<f32>>) = 11,
  Timestamp(i64) = 12,
}

impl<'a> Default for SerializedCell<'a> {
  fn default() -> Self {
    SerializedCell::Null
  }
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
  /// Converts arrow column array to Vec of SerializedCell
  pub fn column_array_to_vec<'b>(
    table_name: &str,
    column: &Column,
    array: &'b ArrayRef,
  ) -> Result<Vec<SerializedCell<'b>>> {
    if !column.nullable && array.null_count() > 0 {
      return Err(null_constraint_violation(table_name, &column.name));
    }

    Ok(match &column.data_type {
      DataType::Boolean => as_boolean_array(array)
        .iter()
        .map(|v| v.map(|v| SerializedCell::Boolean(v)).unwrap_or_default())
        .collect(),
      DataType::Int16 => as_primitive_array::<Int16Type>(array)
        .iter()
        .map(|v| v.map(|v| SerializedCell::Int16(v)).unwrap_or_default())
        .collect(),
      DataType::Int32 => as_primitive_array::<Int32Type>(array)
        .iter()
        .map(|v| v.map(|v| SerializedCell::Int32(v)).unwrap_or_default())
        .collect(),
      DataType::UInt32 => as_primitive_array::<UInt32Type>(array)
        .iter()
        .map(|v| v.map(|v| SerializedCell::UInt32(v)).unwrap_or_default())
        .collect(),
      DataType::Int64 => as_primitive_array::<Int64Type>(array)
        .iter()
        .map(|v| v.map(|v| SerializedCell::Int64(v)).unwrap_or_default())
        .collect(),
      DataType::UInt64 => as_primitive_array::<UInt64Type>(array)
        .iter()
        .map(|v| v.map(|v| SerializedCell::UInt64(v)).unwrap_or_default())
        .collect(),
      DataType::Float32 => as_primitive_array::<Float32Type>(array)
        .iter()
        .map(|v| v.map(|v| SerializedCell::Float32(v)).unwrap_or_default())
        .collect(),
      DataType::Float64 => as_primitive_array::<Float64Type>(array)
        .iter()
        .map(|v| v.map(|v| SerializedCell::Float64(v)).unwrap_or_default())
        .collect(),
      DataType::Varchar { len: _ } | DataType::Text => as_string_array(array)
        .iter()
        .map(|v| {
          v.map(|v| SerializedCell::Blob(v.as_bytes()))
            .unwrap_or_default()
        })
        .collect(),
      DataType::Jsonb => as_string_array(array)
        .iter()
        .map(|v| {
          v.map(|v| SerializedCell::Blob(v.as_bytes()))
            .unwrap_or_default()
        })
        .collect(),
      DataType::Vector { len } => {
        let res: Result<Vec<SerializedCell<'b>>> =
          as_generic_list_array::<i32>(array)
            .iter()
            .map(|maybe_vector| {
              let vector = Arc::new(
                as_primitive_array::<Float32Type>(&maybe_vector.unwrap())
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
            .collect();
        res?
      }
      DataType::Timestamp => {
        let result: Result<Vec<SerializedCell<'b>>> = as_string_array(array)
          .iter()
          .map(|value| {
            value
              .map(|v| {
                let date =
                  chrono::DateTime::parse_from_rfc3339(&v).map_err(|e| {
                    df_error!(Error::InvalidQuery(format!(
                      "Error parsing timestamp: {:?}",
                      e
                    )))
                  })?;
                Ok(SerializedCell::Timestamp(date.timestamp_micros()))
              })
              .unwrap_or_else(|| Ok(SerializedCell::Null))
          })
          .collect();
        result?
      }
      dt => unimplemented!(
        "ColumnArray to Vec<SerializedCell> not implemented for type: {:?}",
        dt
      ),
    })
  }

  // Note: this clones the data, so use it as little as possible
  // This is meant to be used mostly during error generation
  pub fn to_owned(&self) -> OwnedSerializedCell {
    match *self {
      Self::Null => OwnedSerializedCell::Null,
      Self::Boolean(v) => OwnedSerializedCell::Boolean(v),
      Self::Int16(v) => OwnedSerializedCell::Int16(v),
      Self::Int32(v) => OwnedSerializedCell::Int32(v),
      Self::UInt32(v) => OwnedSerializedCell::UInt32(v),
      Self::Int64(v) => OwnedSerializedCell::Int64(v),
      Self::UInt64(v) => OwnedSerializedCell::UInt64(v),
      Self::Float32(v) => OwnedSerializedCell::Float32(v),
      Self::Float64(v) => OwnedSerializedCell::Float64(v),
      Self::Blob(blob) => OwnedSerializedCell::Blob(Arc::new(blob.to_vec())),
      Self::Vector(ref v) => OwnedSerializedCell::Vector(Arc::new(v.to_vec())),
      Self::Timestamp(v) => OwnedSerializedCell::Timestamp(v),
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
      Self::Null => None,
      Self::Boolean(value) => Some(*value),
      _ => self.error_converting_to("boolean"),
    }
  }

  #[inline]
  pub fn as_i16(&self) -> Option<i16> {
    match self {
      Self::Null => None,
      Self::Int16(value) => Some(*value),
      _ => self.error_converting_to("i16"),
    }
  }

  #[inline]
  pub fn as_i32(&self) -> Option<i32> {
    match self {
      Self::Null => None,
      Self::Int32(value) => Some(*value),
      _ => self.error_converting_to("i32"),
    }
  }

  #[inline]
  pub fn as_u32(&self) -> Option<u32> {
    match self {
      Self::Null => None,
      Self::UInt32(value) => Some(*value),
      _ => self.error_converting_to("u32"),
    }
  }

  #[inline]
  pub fn as_i64(&self) -> Option<i64> {
    match self {
      Self::Null => None,
      Self::Int64(value) => Some(*value),
      _ => self.error_converting_to("i64"),
    }
  }

  #[inline]
  pub fn as_u64(&self) -> Option<u64> {
    match self {
      Self::Null => None,
      Self::UInt64(value) => Some(*value),
      _ => self.error_converting_to("u64"),
    }
  }

  #[inline]
  pub fn as_f32(&self) -> Option<f32> {
    match self {
      Self::Null => None,
      Self::Float32(value) => Some(*value),
      _ => self.error_converting_to("f32"),
    }
  }

  #[inline]
  pub fn as_f64(&self) -> Option<f64> {
    match self {
      Self::Null => None,
      Self::Float64(value) => Some(*value),
      _ => self.error_converting_to("f64"),
    }
  }

  #[inline]
  pub fn as_bytes(&self) -> Option<&'a [u8]> {
    match self {
      Self::Null => None,
      Self::Blob(value) => Some(value),
      _ => self.error_converting_to("bytes"),
    }
  }

  #[inline]
  pub fn as_str(&self) -> Option<&'a str> {
    match self {
      Self::Null => None,
      Self::Blob(bytes) => unsafe {
        Some(std::str::from_utf8_unchecked(bytes))
      },
      _ => self.error_converting_to("string"),
    }
  }

  #[inline]
  pub fn as_iso_string(&self) -> Option<String> {
    match self {
      Self::Null => None,
      Self::Timestamp(v) => match NaiveDateTime::from_timestamp_micros(*v) {
        Some(date) => {
          Some(date.and_utc().to_rfc3339_opts(SecondsFormat::Micros, false))
        }
        None => {
          eprintln!("Error converting {:?} to RFC3339 datetime", v);
          None
        }
      },
      _ => self.error_converting_to("iso string"),
    }
  }

  #[inline]
  pub fn as_vector(&self) -> Option<Arc<Vec<f32>>> {
    match self {
      Self::Null => None,
      Self::Vector(v) => Some(v.clone()),
      _ => self.error_converting_to("float vector"),
    }
  }

  fn error_converting_to<T>(&self, ty: &str) -> Option<T> {
    unreachable!("Trying to convert {:?} to {}", &self, &ty);
  }
}
