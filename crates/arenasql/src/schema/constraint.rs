use datafusion::common::Constraint as DfConstraint;
use serde::{Deserialize, Serialize};

use super::proto;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(i32)]
pub enum Constraint {
  /// Columns with the given indices form a composite primary key (they are
  /// jointly unique and not nullable):
  PrimaryKey(Vec<usize>) = 1,
  /// Columns with the given indices form a composite unique key:
  Unique(Vec<usize>) = 2,
}

impl Constraint {
  pub fn from_proto(proto: &proto::Constraint) -> Self {
    let columns = proto.columns.iter().map(|col| *col as usize).collect();
    match proto.r#type {
      1 => Self::PrimaryKey(columns),
      2 => Self::Unique(columns),
      _ => unreachable!(),
    }
  }

  pub fn to_proto(&self) -> proto::Constraint {
    proto::Constraint {
      r#type: self.repr(),
      columns: self.columns().iter().map(|idx| *idx as u32).collect(),
    }
  }

  pub fn repr(&self) -> i32 {
    match self {
      Self::PrimaryKey(_) => 1,
      Self::Unique(_) => 2,
    }
  }

  pub fn columns(&self) -> &Vec<usize> {
    match self {
      Self::PrimaryKey(cols) | Self::Unique(cols) => cols,
    }
  }

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

impl Into<DfConstraint> for &Constraint {
  fn into(self) -> DfConstraint {
    match self {
      Constraint::PrimaryKey(projection) => {
        DfConstraint::PrimaryKey(projection.clone())
      }
      Constraint::Unique(projection) => {
        DfConstraint::Unique(projection.clone())
      }
    }
  }
}
