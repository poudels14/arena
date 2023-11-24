use serde::{Deserialize, Serialize};

use super::Column;

pub type TableId = u16;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Table {
  pub id: TableId,
  pub name: String,
  pub columns: Vec<Column>,
  pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum Constraint {
  /// Columns with the given indices form a composite primary key (they are
  /// jointly unique and not nullable):
  PrimaryKey(Vec<usize>) = 1,
  /// Columns with the given indices form a composite unique key:
  Unique(Vec<usize>) = 2,
}
