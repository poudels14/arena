use serde::{Deserialize, Serialize};

use super::SerializedCell;

#[derive(Debug, Clone)]
pub struct RowId(u64);

impl Default for RowId {
  fn default() -> Self {
    Self(1)
  }
}

impl RowId {
  pub fn add(mut self, value: u64) -> Self {
    self.0 += value;
    self
  }

  pub fn serialize<'a>(&self) -> Vec<u8> {
    self.0.to_be_bytes().to_vec()
  }

  pub fn deserialize(bytes: &[u8]) -> RowId {
    RowId(u64::from_be_bytes(bytes.try_into().unwrap()))
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row<T>(pub Vec<SerializedCell<T>>);

impl<T> Row<T> {
  pub fn project<'a>(
    &'a self,
    columns: &[usize],
  ) -> Vec<&'a SerializedCell<T>> {
    columns.iter().map(|col| &self.0[*col]).collect()
  }
}
