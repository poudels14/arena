use serde::{Deserialize, Serialize};

use super::Constraint;

// Since index id is unique to the table, id doesn't need to be more than 256
pub type TableIndexId = u8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableIndex {
  pub id: TableIndexId,
  pub name: String,
  pub provider: IndexProvider,
}

impl TableIndex {
  #[inline]
  pub fn columns(&self) -> &Vec<usize> {
    self.provider.columns()
  }

  #[inline]
  pub fn is_unique(&self) -> bool {
    self.provider.is_unique()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IndexProvider {
  ColumnIndex {
    columns: Vec<usize>,
    unique: bool,
  },
  HNSWIndex {
    // Must have only one column but need to store as vec to return
    // &Vec<usize> from `Self::columns` method
    columns: Vec<usize>,
  },
}

impl IndexProvider {
  pub fn from_constraint(constraint: &Constraint) -> Self {
    match constraint {
      Constraint::Unique(columns) | Constraint::PrimaryKey(columns) => {
        Self::ColumnIndex {
          columns: columns.to_vec(),
          unique: true,
        }
      }
    }
  }

  #[inline]
  pub fn is_unique(&self) -> bool {
    match self {
      Self::ColumnIndex { unique, .. } => *unique,
      Self::HNSWIndex { .. } => false,
    }
  }

  #[inline]
  pub fn columns(&self) -> &Vec<usize> {
    match self {
      Self::ColumnIndex { columns, .. } => columns,
      Self::HNSWIndex { columns } => columns,
    }
  }
}
