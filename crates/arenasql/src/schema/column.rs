use std::collections::HashMap;

use datafusion::arrow::datatypes::Field;
use serde::{Deserialize, Serialize};

use super::{DataType, SerializedCell};
use crate::Result;

pub type ColumnId = u8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Column {
  pub id: ColumnId,
  pub name: String,
  pub data_type: DataType,
  pub nullable: bool,
  pub default_value: Option<SerializedCell<Vec<u8>>>,
}

impl Column {
  pub fn from_field(idx: ColumnId, field: &Field) -> Result<Self> {
    Ok(Column {
      id: idx,
      name: field.name().to_owned(),
      data_type: DataType::try_from(field.data_type())?,
      nullable: field.is_nullable(),
      default_value: None,
    })
  }

  pub fn to_field(&self, table_name: &str) -> Field {
    Field::new(
      self.name.clone(),
      self.data_type.clone().into(),
      self.nullable,
    )
    .with_metadata(HashMap::from([
      ("table".to_owned(), table_name.to_owned()),
      ("type".to_owned(), self.data_type.to_string()),
    ]))
  }
}
