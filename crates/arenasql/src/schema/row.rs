use serde::{Deserialize, Serialize};

use super::SerializedCell;

pub type RowId = i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct Row<T>(pub Vec<SerializedCell<T>>);
