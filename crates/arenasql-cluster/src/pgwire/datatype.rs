use std::collections::HashMap;

use arenasql::datafusion::{DatafusionDataType, DatafusionField};
use arenasql::pgwire::api::results::{FieldFormat, FieldInfo};
use arenasql::pgwire::api::Type;

pub fn to_field_info(
  field: &DatafusionField,
  field_format: FieldFormat,
) -> FieldInfo {
  FieldInfo::new(
    field.name().clone(),
    None,
    None,
    derive_pg_type(field.data_type(), field.metadata()),
    field_format,
  )
}

pub fn derive_pg_type(
  data_type: &DatafusionDataType,
  metadata: &HashMap<String, String>,
) -> Type {
  match data_type {
    DatafusionDataType::Boolean => Type::BOOL,
    DatafusionDataType::Int32 => Type::INT4,
    DatafusionDataType::UInt32 => Type::INT4,
    DatafusionDataType::Int64 => Type::INT8,
    DatafusionDataType::UInt64 => Type::INT8,
    DatafusionDataType::Float32 => Type::FLOAT4,
    DatafusionDataType::Float64 => Type::FLOAT8,
    DatafusionDataType::Utf8
      if metadata.get("type").map(|t| t.as_str()) == Some("VARCHAR") =>
    {
      Type::VARCHAR
    }
    DatafusionDataType::Utf8 => Type::TEXT,
    DatafusionDataType::Decimal256(76, 1)
      if metadata.get("type").map(|t| t.as_str()) == Some("JSONB") =>
    {
      Type::JSONB
    }
    DatafusionDataType::Decimal256(76, 1) => Type::JSONB,
    // Note: FLOAT4_ARRAY is serialized as JSONB for now :shrug:
    DatafusionDataType::List(_)
      if metadata.get("type").map(|t| t.as_str()) == Some("FLOAT4_ARRAY") =>
    {
      Type::JSONB
    }
    dt => unimplemented!("Type conversion not implemented for: {}", dt),
  }
}
