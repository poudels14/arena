use std::io::Write;
use std::sync::Arc;

use arenasql::arrow::{as_binary_array, as_primitive_array, as_string_array};
use arenasql::bytes::BufMut;
use arenasql::datafusion::DatafusionDataType;
use arenasql::pgwire::api::results::DataRowEncoder;
use arenasql::pgwire::api::Type;
use arenasql::pgwire::error::{PgWireError, PgWireResult};
use arenasql::pgwire::types::ToSqlText;
use arenasql::postgres_types::{IsNull, ToSql};
use arenasql::{arrow, bytes, postgres_types};
use arrow::Array;

use crate::error::ArenaClusterError;

#[macro_export]
macro_rules! encode_all_fields {
  ( $arr_type:ty, $array:ident, $encoders:tt) => {
    $array
      .as_any()
      .downcast_ref::<$arr_type>()
      .unwrap()
      .iter()
      .zip($encoders)
      .map(|(value, encoder)| encoder.encode_field(&value))
      .collect()
  };
}

pub fn encode_column_array(
  encoders: &mut [DataRowEncoder],
  array: &Arc<dyn Array>,
  pg_type: &Type,
) -> PgWireResult<Vec<()>> {
  match array.data_type() {
    DatafusionDataType::Boolean => {
      encode_all_fields!(arrow::BooleanArray, array, encoders)
    }
    DatafusionDataType::Int16 => {
      encode_all_fields!(arrow::Int16Array, array, encoders)
    }
    DatafusionDataType::Int32 => {
      encode_all_fields!(arrow::Int32Array, array, encoders)
    }
    DatafusionDataType::UInt32 => {
      encode_all_fields!(arrow::UInt32Array, array, encoders)
    }
    DatafusionDataType::Int64 => {
      encode_all_fields!(arrow::Int64Array, array, encoders)
    }
    DatafusionDataType::UInt64 => {
      as_primitive_array::<arrow::UInt64Type>(array)
        .iter()
        .zip(encoders)
        .map(|(value, encoder)| encoder.encode_field(&value.map(|v| v as i64)))
        .collect()
    }
    DatafusionDataType::Float32 => {
      encode_all_fields!(arrow::Float32Array, array, encoders)
    }
    DatafusionDataType::Float64 => {
      encode_all_fields!(arrow::Float64Array, array, encoders)
    }
    DatafusionDataType::Binary => match *pg_type {
      Type::JSONB => as_binary_array(array)
        .expect("Unable to downcast to Jsonb binary array")
        .iter()
        .zip(encoders)
        .map(|(value, encoder)| {
          encoder.encode_field(&value.map(|v| SerializedJson(&v)))
        })
        .collect(),
      _ => encode_all_fields!(arrow::BinaryArray, array, encoders),
    },
    // Multiple data types are stored as Utf8 because of datafusion's poor
    // custom data type support. so, do proper conversion here
    DatafusionDataType::Utf8 => match *pg_type {
      Type::TIMESTAMP => as_string_array(array)
        .iter()
        .zip(encoders)
        .map(|(value, encoder)| {
          encoder.encode_field(&value.and_then(|v| {
            match arenasql::chrono::DateTime::parse_from_rfc3339(v) {
              Ok(parsed) => Some(parsed),
              Err(e) => {
                eprintln!("Error parsing timestamp [{}]: {:?}", v, e);
                None
              }
            }
          }))
        })
        .collect(),
      _ => encode_all_fields!(arrow::StringArray, array, encoders),
    },
    DatafusionDataType::List(field) => match pg_type.clone() {
      Type::FLOAT4_ARRAY => array
        .as_any()
        .downcast_ref::<arrow::ListArray>()
        .unwrap()
        .iter()
        .zip(encoders)
        .map(|(arr, encoder)| {
          let float_arr = arr.map(|array| {
            as_primitive_array::<arrow::Float32Type>(&array)
              .iter()
              .map(|v| v.unwrap())
              .collect::<Vec<f32>>()
          });
          encoder.encode_field(&float_arr)
        })
        .collect(),
      ty => {
        unimplemented!(
          "Converting List[{:?}] to {:?} not implemented",
          field,
          ty
        )
      }
    },
    dt => Err(PgWireError::ApiError(Box::new(
      ArenaClusterError::UnsupportedDataType(dt.to_string()),
    ))),
  }
}

/// Json that's already serialized to bytes
#[derive(Debug)]
struct SerializedJson<'a>(&'a [u8]);

impl<'a> ToSqlText for SerializedJson<'a> {
  fn to_sql_text(
    &self,
    _ty: &Type,
    out: &mut bytes::BytesMut,
  ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
  where
    Self: Sized,
  {
    let n = out.writer().write(self.0)?;
    assert_eq!(n, self.0.len());
    Ok(postgres_types::IsNull::No)
  }
}

impl<'a> ToSql for SerializedJson<'a> {
  fn to_sql(
    &self,
    ty: &Type,
    out: &mut bytes::BytesMut,
  ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
  where
    Self: Sized,
  {
    if *ty == Type::JSONB {
      // Need this for binary format
      out.put_u8(1);
    }

    let n = out.writer().write(self.0)?;
    assert_eq!(n, self.0.len());
    Ok(IsNull::No)
  }

  fn to_sql_checked(
    &self,
    ty: &Type,
    out: &mut bytes::BytesMut,
  ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
  {
    self.to_sql(ty, out)
  }

  fn accepts(_ty: &Type) -> bool
  where
    Self: Sized,
  {
    true
  }
}

#[derive(Debug)]
struct Jsonb<'a>(&'a str);

impl<'a> ToSqlText for Jsonb<'a> {
  fn to_sql_text(
    &self,
    _ty: &Type,
    out: &mut bytes::BytesMut,
  ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
  where
    Self: Sized,
  {
    serde_json::ser::to_writer(out.writer(), &self.0)?;
    Ok(postgres_types::IsNull::No)
  }
}

impl<'a> ToSql for Jsonb<'a> {
  fn to_sql(
    &self,
    ty: &Type,
    out: &mut bytes::BytesMut,
  ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
  where
    Self: Sized,
  {
    if *ty == Type::JSONB {
      // Need this for binary format
      out.put_u8(1);
    }
    serde_json::ser::to_writer(out.writer(), &self.0)?;
    Ok(IsNull::No)
  }

  fn to_sql_checked(
    &self,
    ty: &Type,
    out: &mut bytes::BytesMut,
  ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
  {
    self.to_sql(ty, out)
  }

  fn accepts(_ty: &Type) -> bool
  where
    Self: Sized,
  {
    true
  }
}
