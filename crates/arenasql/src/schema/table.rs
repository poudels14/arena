use std::sync::Arc;

use datafusion::datasource::TableProvider as DfTableProvider;
use serde::{Deserialize, Serialize};

use super::{Column, ColumnId, Constraint};
use crate::Result;

pub type TableId = u16;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Table {
  pub id: TableId,
  pub name: String,
  pub columns: Vec<Column>,
  pub constraints: Vec<Constraint>,
}

impl Table {
  pub fn new(
    id: TableId,
    name: &str,
    provider: Arc<dyn DfTableProvider>,
  ) -> Result<Self> {
    let columns = provider
      .schema()
      .fields
      .iter()
      .enumerate()
      .map(|(idx, field)| Column::from_field(idx as ColumnId, field))
      .collect::<Result<Vec<Column>>>()?;

    Ok(Table {
      id,
      name: name.to_owned(),
      columns,
      constraints: provider
        .constraints()
        .map(|constraints| {
          constraints
            .as_ref()
            .into_iter()
            .map(|c| Constraint::from(c))
            .collect()
        })
        .unwrap_or_default(),
    })
  }
}
