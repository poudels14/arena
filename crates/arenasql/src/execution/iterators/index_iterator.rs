use crate::execution::filter::Filter;
use crate::schema::{DataFrame, Row, RowId, SerializedCell, Table, TableIndex};
use crate::storage::{KeyValueGroup, KeyValueIterator, StorageHandler};
use crate::{
  index_row_key, index_rows_prefix_key, table_row_key, Error, Result,
};

#[allow(unused)]
pub struct IndexIterator<'a> {
  storage: &'a StorageHandler,
  table: &'a Table,
  index: &'a TableIndex,
  column_projection: &'a Vec<usize>,
  filters: &'a Vec<Filter>,
}

impl<'a> IndexIterator<'a> {
  pub fn new(
    storage: &'a StorageHandler,
    table: &'a Table,
    index: &'a TableIndex,
    filters: &'a Vec<Filter>,
    column_projection: &'a Vec<usize>,
  ) -> IndexIterator<'a> {
    Self {
      storage,
      table,
      index,
      column_projection,
      filters,
    }
  }

  pub fn fill_into(&self, dataframe: &mut DataFrame) -> Result<()> {
    let index_scan_prefix_row = self.select_eq_filters_for_prefix();
    let index_scan_prefix =
      self.generate_index_scan_prefix(&index_scan_prefix_row)?;
    let _scan_filters = self
      .filters
      .split_at(index_scan_prefix_row.len())
      .1
      .to_vec();

    let mut index_iter = self
      .storage
      .kv
      .scan_with_prefix(KeyValueGroup::IndexRows, &index_scan_prefix)?;

    match (self.index.is_unique(), self.requires_table_lookup()) {
      (true, false) => {
        self.scan_unique_index_into_df(&mut index_iter, dataframe)
      }
      (true, true) => self.scan_unique_index_with_table_lookup_into_df(
        &mut index_iter,
        dataframe,
      ),
      (false, false) => {
        self.scan_secondary_index_into_df(&mut index_iter, dataframe)
      }
      (false, true) => {
        unimplemented!()
      }
    }
  }

  /// This scans the unique index and fills the dataframe
  /// This ONLY handles case where unique index is used and
  /// all selected columns are present in the index
  fn scan_unique_index_into_df(
    &self,
    index_iter: &mut Box<dyn KeyValueIterator>,
    dataframe: &mut DataFrame,
  ) -> Result<()> {
    let index_prefix = index_rows_prefix_key!(self.index.id);
    let projection_on_index_columns =
      self.valid_index_columns_projection(self.column_projection);
    while let Some((index_row_with_prefix, row_id)) = index_iter.get() {
      let index_row_bytes = &index_row_with_prefix[index_prefix.len()..];
      let index_columns = self
        .storage
        .serializer
        .deserialize::<Vec<SerializedCell<'_>>>(index_row_bytes)?;

      let selected_columns = projection_on_index_columns
        .iter()
        .map(|proj| &index_columns[*proj])
        .collect();

      dataframe.append_row(row_id, &selected_columns);
      index_iter.next();
    }
    Ok(())
  }

  /// This scans the non-unique index and fills the dataframe
  /// This ONLY handles case where non-unique index is used and
  /// all selected columns are present in the index
  fn scan_secondary_index_into_df(
    &self,
    index_iter: &mut Box<dyn KeyValueIterator>,
    dataframe: &mut DataFrame,
  ) -> Result<()> {
    let index_prefix = index_rows_prefix_key!(self.index.id);
    let projection_on_index_columns =
      self.valid_index_columns_projection(self.column_projection);
    while let Some((index_row_with_prefix, _)) = index_iter.get() {
      let index_row_bytes = &index_row_with_prefix[index_prefix.len()..];
      let (index_columns, row_id) =
        self
          .storage
          .serializer
          .deserialize::<(Vec<SerializedCell<'_>>, &[u8])>(index_row_bytes)?;

      let selected_columns = projection_on_index_columns
        .iter()
        .map(|proj| &index_columns[*proj])
        .collect();

      dataframe.append_row(row_id, &selected_columns);
      index_iter.next();
    }
    Ok(())
  }

  /// This scans the unique index and looks up the table rows to
  /// get columns that are not present in the index. This handles
  /// the case where unique index is used but required looking up
  /// the table because the index doesn't have all the columns selected
  /// by the query
  fn scan_unique_index_with_table_lookup_into_df(
    &self,
    index_iter: &mut Box<dyn KeyValueIterator>,
    dataframe: &mut DataFrame,
  ) -> Result<()> {
    while let Some((_, row_id)) = index_iter.get() {
      let row_bytes = self
        .storage
        .kv
        .get(KeyValueGroup::Rows, &table_row_key!(self.table.id, &row_id))?
        .ok_or_else(|| {
          Error::IOError(format!(
            "Couldn't find row data for rowid: {:?}",
            RowId::deserialize(&row_id)
          ))
        })?;

      let row = self.storage.serializer.deserialize::<Row<'_>>(&row_bytes)?;

      let selected_columns = self
        .column_projection
        .iter()
        .map(|proj| &row[*proj])
        .collect();

      dataframe.append_row(&row_id, &selected_columns);
      index_iter.next();
    }
    Ok(())
  }

  #[inline]
  pub fn requires_table_lookup(&self) -> bool {
    self
      .valid_index_columns_projection(self.column_projection)
      .len()
      != self.column_projection.len()
  }

  /// Returns a list of position on the index columns for
  /// all columns found in the index. If the column isn't
  /// in the index, it's excluded from the return value
  /// For example, if the index has columns [2, 3], then
  /// this returns [0] for input [2] and returns [1] for
  /// input [3, 4] since only column idx 3 is present in the
  /// index at position 1
  pub fn valid_index_columns_projection(
    &self,
    column_projection: &Vec<usize>,
  ) -> Vec<usize> {
    column_projection
      .iter()
      .filter_map(|col| {
        self
          .index
          .columns()
          .iter()
          .position(|idx_col| idx_col == col)
      })
      .collect()
  }

  /// returns a row with columns that can be used to generate a prefix
  /// key to iterate over the key value store. Only the columns with
  /// exact match filter are returned and they are in the same order
  /// as they appear in the index such that concatenating the returned
  /// rows will make a valid prefix (as long as it's serialized properly
  /// to match the prefix when if the returned row doesn't contain all the
  /// columns used in the index. For example, bincode serialization adds
  /// the vec length at the begining of the serialized value, so that should
  /// be changed to match the number of index columns if the returned
  /// row doesn't have all the columns in the index)
  fn select_eq_filters_for_prefix(&'a self) -> Vec<SerializedCell<'a>> {
    self
      .index
      .columns()
      .iter()
      .map(|col| {
        self.filters.iter().find_map(|filter| {
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
      .collect::<Vec<SerializedCell<'a>>>()
  }

  fn generate_index_scan_prefix<'b>(
    &self,
    eq_filter_for_prefix: &Vec<SerializedCell<'b>>,
  ) -> Result<Vec<u8>> {
    match eq_filter_for_prefix.len() > 0 {
      true => {
        let mut serialized_prefix_filters = self
          .storage
          .serializer
          .serialize::<Vec<SerializedCell<'b>>>(&eq_filter_for_prefix)?;
        // When the Vec<cell> is serialized, first byte(s) is the length of
        // the Vec. So, if the index being used is a composite index but the
        // number of `=` filters being used is less than that length, then
        // prefix doesn't match. So, update the first byte of the serialized
        // filter
        let length_byte = self
          .storage
          .serializer
          .serialize(&self.index.columns().len())?;

        // Note: don't expect number of columns in the index to be more than
        // 20, so it can definitely fit in the first byte [value = ~200]
        // otherwise, panic!
        assert_eq!(
          length_byte.len(),
          1,
          "Number of column in the index exceeds the MAX supported [~230]"
        );
        serialized_prefix_filters[0] = length_byte[0];

        Ok(index_row_key!(self.index.id, &serialized_prefix_filters))
      }
      false => Ok(index_rows_prefix_key!(self.index.id)),
    }
  }
}
