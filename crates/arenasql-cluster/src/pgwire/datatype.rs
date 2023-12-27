use std::collections::HashMap;
use std::str::FromStr;

use arenasql::datafusion::{DatafusionDataType, DatafusionField};
use arenasql::pgwire::api::results::{FieldFormat, FieldInfo};
use arenasql::pgwire::api::Type;
use arenasql::schema::DataType as ArenaDataType;

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
  // If there's metadata, use it, else derive default type
  // Metadata will be set if the type came from table schema
  match metadata.get("TYPE") {
    Some(ty) => ArenaDataType::from_str(ty).unwrap().pg_type(),
    // The following is to derive fields that aren't associated with
    // a table, for sth like scalar value
    _ => match data_type {
      DatafusionDataType::Boolean => Type::BOOL,
      DatafusionDataType::Int32 => Type::INT4,
      DatafusionDataType::UInt32 => Type::INT4,
      DatafusionDataType::Int64 => Type::INT8,
      DatafusionDataType::UInt64 => Type::INT8,
      DatafusionDataType::Float32 => Type::FLOAT4,
      DatafusionDataType::Float64 => Type::FLOAT8,
      DatafusionDataType::Utf8 => Type::TEXT,
      DatafusionDataType::Decimal256(_, _) => Type::JSONB,
      DatafusionDataType::Timestamp(_, _) => Type::TIMESTAMP,
      DatafusionDataType::List(_) => Type::JSONB,
      dt => unimplemented!("Type conversion not implemented for: {}", dt),
    },
  }
}
