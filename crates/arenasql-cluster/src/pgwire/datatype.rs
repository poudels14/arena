use std::collections::HashMap;
use std::sync::Arc;

use arenasql::records::{self, DatafusionDataType};
use pgwire::api::results::{FieldFormat, FieldInfo};
use pgwire::api::Type;

pub fn to_field_info(field: &Arc<records::DatafusionField>) -> FieldInfo {
  let (pg_type, field_format) =
    derive_pg_type(field.data_type(), field.metadata());
  FieldInfo::new(field.name().clone(), None, None, pg_type, field_format)
}

fn derive_pg_type(
  data_type: &DatafusionDataType,
  metadata: &HashMap<String, String>,
) -> (Type, FieldFormat) {
  match data_type {
    DatafusionDataType::Boolean => (Type::BOOL, FieldFormat::Text),
    DatafusionDataType::Int32 => (Type::INT4, FieldFormat::Text),
    DatafusionDataType::Int64 => (Type::INT8, FieldFormat::Text),
    DatafusionDataType::UInt64 => (Type::INT8, FieldFormat::Text),
    DatafusionDataType::Float32 => (Type::FLOAT4, FieldFormat::Text),
    DatafusionDataType::Float64 => (Type::FLOAT8, FieldFormat::Text),
    DatafusionDataType::Utf8 => {
      match metadata.get("type").map(|t| t.as_str()) {
        Some("JSONB") => (Type::JSONB, FieldFormat::Text),
        _ => (Type::TEXT, FieldFormat::Text),
      }
    }
    dt => unimplemented!("Type conversion not implemented for: {}", dt),
  }
}
