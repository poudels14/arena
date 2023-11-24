use datafusion::arrow::datatypes::Field;
use serde::{Deserialize, Serialize};

use super::{DataType, DataWithValue};

pub type ColumnId = u16;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Column {
  pub id: ColumnId,
  pub name: String,
  pub data_type: DataType,
  pub nullable: bool,
  pub default_value: Option<DataWithValue<Vec<u8>>>,
}

impl Column {
  pub fn from_field(idx: u16, field: &Field) -> Self {
    Column {
      id: idx,
      name: field.name().to_owned(),
      data_type: DataType::try_from(field.data_type()).unwrap(),
      nullable: field.is_nullable(),
      default_value: None,
    }
  }

  pub fn to_field(&self) -> Field {
    Field::new(
      self.name.clone(),
      self.data_type.clone().into(),
      self.nullable,
    )
  }
}
