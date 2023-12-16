use datafusion::arrow::datatypes::Field;
use derive_new::new;
use serde::{Deserialize, Serialize};

use super::{DataType, OwnedSerializedCell};
use crate::Result;

pub type ColumnId = u8;

#[derive(Debug, Clone, Serialize, new, Deserialize, PartialEq)]
pub struct Column {
  pub id: ColumnId,
  pub name: String,
  pub data_type: DataType,
  pub nullable: bool,
  pub default_value: Option<OwnedSerializedCell>,
}

impl Column {
  pub fn from_field(idx: ColumnId, field: &Field) -> Result<Self> {
    Ok(Column {
      id: idx,
      name: field.name().to_owned(),
      data_type: DataType::from_field(field)?,
      nullable: field.is_nullable(),
      default_value: None,
    })
  }

  pub fn to_field(&self, table_name: &str) -> Field {
    let (data_type, mut metadata) = self.data_type.to_df_datatype();
    metadata.extend([
      ("table".to_owned(), table_name.to_owned()),
      ("type".to_owned(), self.data_type.to_string()),
    ]);
    Field::new(self.name.clone(), data_type, self.nullable)
      .with_metadata(metadata)
  }
}
