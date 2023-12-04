use derive_new::new;

use crate::schema::{RowId, TableId};

#[derive(Debug, Clone, Eq, PartialEq, new)]
pub struct ModifiedRow {
  pub table_id: TableId,
  pub row_id: RowId,
  pub mod_type: ModificationType,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum ModificationType {
  INSERT,
  UPDATE,
  DELETE,
}
