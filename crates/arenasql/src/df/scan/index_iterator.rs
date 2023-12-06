use super::filter::Filter;
use crate::schema::{SerializedCell, Table, TableIndex};
use crate::storage::{KeyValueGroup, RowIterator, StorageOperator};
use crate::{index_rows_key, table_rows_prefix_key, Result};

#[allow(unused)]
pub(super) struct UniqueIndexIterator<'a> {
  table: &'a Table,
  index: &'a TableIndex,
  filters: &'a Vec<Filter>,
  storage: &'a StorageOperator,
  index_iter: Box<dyn RowIterator>,
  next_key_value: Option<(Vec<u8>, Vec<u8>)>,
}

impl<'a> UniqueIndexIterator<'a> {
  pub fn new(
    table: &'a Table,
    index: &'a TableIndex,
    filters: &'a Vec<Filter>,
    storage: &'a StorageOperator,
  ) -> Result<Self> {
    let projected_cells = index
      .columns
      .iter()
      .map(|col| {
        filters
          .iter()
          .find(|filter| {
            filter.is_eq()
              && filter
                .get_column_projection()
                .iter()
                .any(|col_proj| *col_proj == *col)
          })
          .and_then(|filter| filter.get_binary_eq_literal())
      })
      .take_while(|lit| lit.is_some())
      .map(|v| v.unwrap())
      .collect::<Vec<SerializedCell<Vec<u8>>>>();

    let serialized_projected_cells =
      storage
        .serializer
        .serialize::<Vec<SerializedCell<Vec<u8>>>>(&projected_cells)?;

    let index_iter = storage.kv.scan_with_prefix(
      KeyValueGroup::Indexes,
      &index_rows_key!(index.id, &serialized_projected_cells),
    )?;

    let mut unique_iterator = UniqueIndexIterator {
      table,
      index,
      filters,
      storage,
      index_iter,
      next_key_value: None,
    };
    unique_iterator.next();
    Ok(unique_iterator)
  }
}

impl<'a> RowIterator for UniqueIndexIterator<'a> {
  fn key(&self) -> Option<&[u8]> {
    self.index_iter.key()
  }

  fn get(&self) -> Option<(&[u8], &[u8])> {
    if self.next_key_value.is_some() {
      self
        .next_key_value
        .as_ref()
        .map(|o| (o.0.as_slice(), o.1.as_slice()))
    } else {
      None
    }
  }

  fn next(&mut self) {
    if let Some((key, row_id)) = self.index_iter.get() {
      let next_value = self
        .storage
        .kv
        .get(
          KeyValueGroup::Rows,
          &vec![table_rows_prefix_key!(self.table.id).as_slice(), &row_id]
            .concat(),
        )
        .unwrap();
      self.next_key_value =
        next_value.and_then(|value| Some((key.to_vec(), value)))
    } else {
      self.next_key_value = None;
    }
    self.index_iter.next()
  }
}
