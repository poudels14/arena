use serde::{Deserialize, Serialize};

// Since index id is unique to the table, id doesn't need to be more than 256
pub type TableIndexId = u8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableIndex {
  pub id: TableIndexId,
  pub name: String,
  pub columns: Vec<usize>,
  pub allow_duplicates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IndexType {
  Unique(Vec<usize>),
  NonUnique(Vec<usize>),
}
