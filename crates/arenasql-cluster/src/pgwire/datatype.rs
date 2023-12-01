use std::sync::Arc;

use arenasql::records::{self, DatafusionDataType};
use pgwire::api::results::{FieldFormat, FieldInfo};
use pgwire::api::Type;

pub fn to_field_info(field: &Arc<records::DatafusionField>) -> FieldInfo {
  FieldInfo::new(
    field.name().clone(),
    None,
    None,
    to_postgres_type(field.data_type()),
    FieldFormat::Text,
  )
}

fn to_postgres_type(data_type: &DatafusionDataType) -> Type {
  match data_type {
    DatafusionDataType::Boolean => Type::BOOL,
    DatafusionDataType::Int32 => Type::INT4,
    DatafusionDataType::Int64 => Type::INT8,
    DatafusionDataType::UInt64 => Type::INT8,
    DatafusionDataType::Float32 => Type::FLOAT4,
    DatafusionDataType::Float64 => Type::FLOAT8,
    DatafusionDataType::Utf8 => Type::TEXT,
    dt => unimplemented!("Type conversion not implemented for: {}", dt),
  }
}
