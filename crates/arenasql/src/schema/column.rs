use bitflags::bitflags;
use datafusion::arrow::datatypes::Field;
use derive_new::new;
use serde::{Deserialize, Serialize};

use super::{DataType, OwnedSerializedCell, Table};
use crate::Result;

pub static CTID_COLUMN: &'static str = "ctid";
pub type ColumnId = u8;

bitflags! {
  #[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Serialize,
    Deserialize,
  )]
  pub struct ColumnProperty: u16 {
    const DEFAULT = 0;
    const ARCHIVED = 1 << 1;
    const NOT_NULL = 1 << 2;
    const UNIQUE = 1 << 3;
  }
}

impl Default for ColumnProperty {
  fn default() -> Self {
    Self::DEFAULT
  }
}

#[derive(Debug, Clone, Serialize, new, Deserialize, PartialEq)]
pub struct Column {
  pub id: ColumnId,
  pub name: String,
  pub data_type: DataType,
  pub properties: ColumnProperty,
  pub default_value: Option<OwnedSerializedCell>,
}

impl Column {
  pub fn from_field(idx: ColumnId, field: &Field) -> Result<Self> {
    let mut properties = ColumnProperty::default();
    if !field.is_nullable() {
      properties = properties | ColumnProperty::NOT_NULL;
    }
    Ok(Column {
      id: idx,
      name: field.name().to_owned(),
      data_type: DataType::from_field(field)?,
      properties,
      default_value: None,
    })
  }

  pub fn nullable(&self) -> bool {
    !self.properties.intersects(ColumnProperty::NOT_NULL)
  }

  pub fn unique(&self) -> bool {
    self.properties.intersects(ColumnProperty::UNIQUE)
  }

  pub fn to_field(&self, table: &Table) -> Field {
    let (data_type, mut metadata) = self.data_type.to_df_datatype();
    metadata.insert("TABLE_NAME".to_owned(), table.name.to_owned());
    metadata.insert("TABLE_ID".to_owned(), table.id.to_string());
    Field::new(self.name.clone(), data_type, self.nullable())
      .with_metadata(metadata)
  }
}
