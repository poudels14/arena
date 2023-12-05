use serde::{Deserialize, Serialize};

use super::SerializedCell;

pub type RowId = u64;

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
