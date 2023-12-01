use std::sync::Arc;

use arenasql::arrow;
use arenasql::records::DatafusionDataType;
use arrow::Array;
use pgwire::api::results::DataRowEncoder;
use pgwire::error::{PgWireError, PgWireResult};

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
      dt => Err(PgWireError::ApiError(Box::new(
        ArenaClusterError::UnsupportedDataType(dt.to_string()),
      ))),
    }
  }
}
