use super::filter::Filter;
use crate::schema::{SerializedCell, Table, TableIndex};
use crate::storage::{KeyValueGroup, RowIterator, StorageOperator};
use crate::{
  index_row_key, index_rows_prefix_key, table_rows_prefix_key, Result,
};

#[allow(unused)]
pub struct UniqueIndexIterator<'a> {
  table: &'a Table,
  index: &'a TableIndex,
  storage: &'a StorageOperator,
  index_prefix: Vec<u8>,
  index_iter: Box<dyn RowIterator>,
  scan_filters: Option<Vec<Filter>>,
  next_key_value: Option<(Vec<u8>, Vec<u8>)>,
}

pub fn new<'a>(
  table: &'a Table,
  index: &'a TableIndex,
  filters: &'a Vec<Filter>,
  storage: &'a StorageOperator,
) -> Result<Box<UniqueIndexIterator<'a>>> {
  let prefix_filters = index
    .columns
    .iter()
    .map(|col| {
      filters.iter().find_map(|filter| {
        if filter
          .get_column_projection()
          .iter()
          .any(|col_proj| *col_proj == *col)
        {
          filter.get_binary_eq_literal()
        } else {
          None
        }
      })
    })
    .take_while(|lit| lit.is_some())
    .map(|v| v.unwrap())
    .collect::<Vec<SerializedCell<Vec<u8>>>>();

  let prefix = match prefix_filters.len() > 0 {
    true => {
      let mut serialized_prefix_filters =
        storage
          .serializer
          .serialize::<Vec<SerializedCell<Vec<u8>>>>(&prefix_filters)?;
      // When the Vec<cell> is serialized, first byte(s) is the length of
      // the Vec. So, if the index being used is a composite index but the
      // number of `=` filters being used is less than that length, then
      // prefix doesn't match. So, update the first byte of the serialized
      // filter
      let length_byte = storage.serializer.serialize(&index.columns.len())?;

      // Note: don't expect number of columns in the index to be more than
      // 20, so it can definitely fit in the first byte [value = ~200]
      // otherwise, panic!
      assert_eq!(
        length_byte.len(),
        1,
        "Number of column in the index exceeds the MAX supported"
      );
      serialized_prefix_filters[0] = length_byte[0];

      index_row_key!(index.id, &serialized_prefix_filters)
    }
    false => {
      index_rows_prefix_key!(index.id)
    }
  };

  let index_iter = storage
    .kv
    .scan_with_prefix(KeyValueGroup::Indexes, &prefix)?;

  let scan_filters = filters.split_at(prefix_filters.len()).1.to_vec();

  let mut unique_iterator = UniqueIndexIterator {
    table,
    index,
    storage,
    index_prefix: index_rows_prefix_key!(index.id),
    index_iter,
    scan_filters: if scan_filters.len() > 0 {
      Some(scan_filters)
    } else {
      None
    },
    next_key_value: None,
  };
  unique_iterator.next();
  Ok(Box::new(unique_iterator))
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
      // let serialized_index_row = &key[self.index_prefix.len()..];

      // let index_row = self
      //   .storage
      //   .serializer
      //   .deserialize::<Vec<SerializedCell<&[u8]>>>(&serialized_index_row)
      // .unwrap();

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
