use datafusion::common::Constraint as DfConstraint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Constraint {
  /// Columns with the given indices form a composite primary key (they are
  /// jointly unique and not nullable):
  PrimaryKey(Vec<usize>),
  /// Columns with the given indices form a composite unique key:
  Unique(Vec<usize>),
}

impl Constraint {
  pub fn needs_index(&self) -> bool {
    match self {
      Self::PrimaryKey(_) => true,
      Self::Unique(_) => true,
    }
  }
}

impl From<&DfConstraint> for Constraint {
  fn from(value: &DfConstraint) -> Self {
    match value {
      DfConstraint::PrimaryKey(projection) => {
        Self::PrimaryKey(projection.clone())
      }
      DfConstraint::Unique(projection) => Self::Unique(projection.clone()),
    }
  }
}
