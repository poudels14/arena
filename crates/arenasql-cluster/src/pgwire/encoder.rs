use std::sync::Arc;

use arenasql::bytes::BufMut;
use arenasql::postgres_types::{IsNull, ToSql};
use arenasql::records::DatafusionDataType;
use arenasql::{arrow, bytes, postgres_types};
use arrow::Array;
use pgwire::api::results::DataRowEncoder;
use pgwire::api::Type;
use pgwire::error::{PgWireError, PgWireResult};
use pgwire::types::ToSqlText;
use serde_json::json;

use crate::error::ArenaClusterError;

pub trait ColumnEncoder {
  fn encode_column_array(
    &self,
    encoders: &mut [DataRowEncoder],
  ) -> PgWireResult<Vec<()>>;
}

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

impl<'a> ColumnEncoder for &Arc<dyn Array> {
  fn encode_column_array(
    &self,
    encoders: &mut [DataRowEncoder],
  ) -> PgWireResult<Vec<()>> {
    match self.data_type() {
      DatafusionDataType::Boolean => {
        encode_all_fields!(arrow::BooleanArray, self, encoders)
      }
      DatafusionDataType::Binary => {
        encode_all_fields!(arrow::BinaryArray, self, encoders)
      }
      DatafusionDataType::Int32 => {
        encode_all_fields!(arrow::Int32Array, self, encoders)
      }
      DatafusionDataType::Int64 => {
        encode_all_fields!(arrow::Int64Array, self, encoders)
      }
      DatafusionDataType::Float32 => {
        encode_all_fields!(arrow::Float32Array, self, encoders)
      }
      DatafusionDataType::Float64 => {
        encode_all_fields!(arrow::Float64Array, self, encoders)
      }
      DatafusionDataType::Utf8 => {
        encode_all_fields!(arrow::StringArray, self, encoders)
      }
      DatafusionDataType::List(_) => self
        .as_any()
        .downcast_ref::<arrow::ListArray>()
        .unwrap()
        .iter()
        .zip(encoders)
        .map(|(value, encoder)| {
          let float_arr = value.map(|v| {
            FloatArray(
              v.as_any()
                .downcast_ref::<arrow::Float32Array>()
                .unwrap()
                .iter()
                .map(|v| v.unwrap())
                .collect::<Vec<f32>>(),
            )
          });
          encoder.encode_field_with_type_and_format(
            &float_arr,
            &Type::JSONB,
            pgwire::api::results::FieldFormat::Text,
          )
        })
        .collect(),
      dt => Err(PgWireError::ApiError(Box::new(
        ArenaClusterError::UnsupportedDataType(dt.to_string()),
      ))),
    }
  }
}

#[derive(Debug)]
struct FloatArray(Vec<f32>);

impl ToSqlText for FloatArray {
  fn to_sql_text(
    &self,
    _ty: &Type,
    out: &mut bytes::BytesMut,
  ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
  where
    Self: Sized,
  {
    serde_json::ser::to_writer(out.writer(), &json!(self.0))?;
    Ok(postgres_types::IsNull::No)
  }
}

impl ToSql for FloatArray {
  fn to_sql(
    &self,
    _ty: &Type,
    out: &mut bytes::BytesMut,
  ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
  where
    Self: Sized,
  {
    serde_json::ser::to_writer(out.writer(), &json!(self.0))?;
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
