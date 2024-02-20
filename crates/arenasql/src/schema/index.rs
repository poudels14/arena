use strum_macros::{EnumString, FromRepr};

use super::{proto, Constraint};

/// Table index id is unique to the database
pub type TableIndexId = u16;

#[derive(Debug, Clone, PartialEq)]
pub struct TableIndex {
  pub id: TableIndexId,
  pub name: String,
  pub provider: IndexProvider,
}

impl TableIndex {
  pub fn from_proto(index: &proto::TableIndex) -> Self {
    TableIndex {
      id: index.id as u16,
      name: index.name.clone(),
      provider: match index.provider.as_ref().unwrap() {
        proto::TableIndexProvider::Basic(provider) => {
          IndexProvider::BasicIndex {
            columns: provider.columns.iter().map(|col| *col as usize).collect(),
            unique: provider.unique,
          }
        }
        proto::TableIndexProvider::Hnsw(provider) => IndexProvider::HNSWIndex {
          columns: provider.columns.iter().map(|col| *col as usize).collect(),
          metric: VectorMetric::from_repr(provider.metric as usize).unwrap(),
          m: provider.m as usize,
          ef_construction: provider.ef_construction as usize,
          ef: provider.ef as usize,
          dim: provider.dim as usize,
          retain_vectors: provider.retain_vectors.unwrap_or(false),
          namespace_column: provider.namespace_column.map(|idx| idx as usize),
        },
      },
    }
  }

  pub fn to_proto(&self) -> proto::TableIndex {
    proto::TableIndex {
      id: self.id as u32,
      name: self.name.clone(),
      provider: Some(match &self.provider {
        IndexProvider::BasicIndex { columns, unique } => {
          proto::TableIndexProvider::Basic(proto::BasicIndexProvider {
            columns: columns.iter().map(|c| *c as u32).collect(),
            unique: *unique,
          })
        }
        IndexProvider::HNSWIndex {
          columns,
          metric,
          m,
          ef_construction,
          ef,
          dim,
          retain_vectors,
          namespace_column,
        } => proto::TableIndexProvider::Hnsw(proto::HnswIndexProvider {
          columns: columns.iter().map(|c| *c as u32).collect(),
          metric: metric.clone() as i32,
          m: *m as u32,
          ef_construction: *ef_construction as u32,
          ef: *ef as u32,
          dim: *dim as u32,
          retain_vectors: Some(*retain_vectors),
          namespace_column: namespace_column.map(|idx| idx as u32),
        }),
      }),
    }
  }

  #[inline]
  pub fn columns(&self) -> &Vec<usize> {
    self.provider.columns()
  }

  #[inline]
  pub fn is_unique(&self) -> bool {
    self.provider.is_unique()
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IndexProvider {
  BasicIndex {
    columns: Vec<usize>,
    unique: bool,
  },
  HNSWIndex {
    // Must have only one column but need to store as vec to return
    // &Vec<usize> from `Self::columns` method
    columns: Vec<usize>,
    metric: VectorMetric,
    m: usize,
    ef_construction: usize,
    ef: usize,
    dim: usize,
    // by default, set this to false
    // if set to false, flat vectors of the columns will be cleared out
    // for the rows that are already indexed; this is to avoid having
    // duplicate vectors in index as well as row data and save space.
    // storing embedding vectors is expensive!
    retain_vectors: bool,
    // column to split the indexing by
    namespace_column: Option<usize>,
  },
}

#[derive(Debug, Clone, PartialEq, FromRepr, EnumString)]
pub enum VectorMetric {
  /// Dot product
  #[strum(ascii_case_insensitive)]
  Dot = 1,
  /// L2 squared
  #[strum(ascii_case_insensitive)]
  L2 = 2,
  /// Cosine distance
  #[strum(ascii_case_insensitive)]
  Cos = 3,
}

impl IndexProvider {
  pub fn from_constraint(constraint: &Constraint) -> Self {
    match constraint {
      Constraint::Unique(columns) | Constraint::PrimaryKey(columns) => {
        Self::BasicIndex {
          columns: columns.to_vec(),
          unique: true,
        }
      }
    }
  }

  #[inline]
  pub fn is_unique(&self) -> bool {
    match self {
      Self::BasicIndex { unique, .. } => *unique,
      Self::HNSWIndex { .. } => false,
    }
  }

  #[inline]
  pub fn columns(&self) -> &Vec<usize> {
    match self {
      Self::BasicIndex { columns, .. } => columns,
      Self::HNSWIndex { columns, .. } => columns,
    }
  }
}
