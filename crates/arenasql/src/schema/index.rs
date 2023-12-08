use serde::{Deserialize, Serialize};

use super::Constraint;

// Since index id is unique to the table, id doesn't need to be more than 256
pub type TableIndexId = u8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableIndex {
  pub id: TableIndexId,
  pub name: String,
  pub index_type: IndexType,
}

impl TableIndex {
  pub fn columns(&self) -> &Vec<usize> {
    self.index_type.columns()
  }

  pub fn is_unique(&self) -> bool {
    self.index_type.is_unique()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IndexType {
  Unique(Vec<usize>),
  NonUnique(Vec<usize>),
}

impl IndexType {
  pub fn from_constraint(constraint: &Constraint) -> Self {
    match constraint {
      Constraint::Unique(columns) | Constraint::PrimaryKey(columns) => {
        Self::Unique(columns.to_vec())
      }
    }
  }

  #[inline]
  pub fn is_unique(&self) -> bool {
    match self {
      Self::Unique(..) => true,
      _ => false,
    }
  }

  #[inline]
  pub fn columns(&self) -> &Vec<usize> {
    match self {
      Self::Unique(columns) | Self::NonUnique(columns) => columns,
    }
  }
}
