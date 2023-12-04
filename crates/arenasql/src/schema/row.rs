use serde::{Deserialize, Serialize};

use super::SerializedCell;

pub type RowId = u64;

#[derive(Debug, Serialize, Deserialize)]
pub struct Row<T>(pub Vec<SerializedCell<T>>);
