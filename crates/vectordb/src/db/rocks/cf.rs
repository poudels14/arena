use super::PinnableSlice;
use crate::storage::Database;
use anyhow::{anyhow, Result};
use rocksdb::{
  ColumnFamily, DBIteratorWithThreadMode, Direction, IteratorMode, ReadOptions,
  WriteBatchWithTransaction,
};

pub type RowsIterator<'a> = DBIteratorWithThreadMode<'a, Database>;

pub fn column_handle<'a>(
  db: &'a Database,
  name: &str,
) -> Result<&'a ColumnFamily> {
  db.cf_handle(name)
    .ok_or(anyhow!("Failed to get handle for column: {}", name))
}

pub trait DatabaseColumnFamily<'a> {
  fn get_pinned(&self, key: &[u8]) -> Result<Option<PinnableSlice>>;

  fn get_pinned_opt<K>(
    &self,
    key: K,
    options: &ReadOptions,
  ) -> Result<Option<PinnableSlice>>
  where
    K: AsRef<[u8]>;

  fn put(&self, key: &[u8], value: &[u8]) -> Result<()>;

  fn batch_put(
    &self,
    batch: &mut WriteBatchWithTransaction<true>,
    key: &[u8],
    value: &[u8],
  );

  fn batch_delete(
    &self,
    batch: &mut WriteBatchWithTransaction<true>,
    key: &[u8],
  );

  fn iterator(&self, mode: IteratorMode) -> RowsIterator;

  fn iterator_opt(
    &self,
    options: ReadOptions,
    mode: IteratorMode,
  ) -> RowsIterator;

  fn prefix_iterator(&self, prefix: &[u8]) -> RowsIterator;

  #[allow(unused_variables)]
  fn prefix_iterator_opt(
    &self,
    prefix: &[u8],
    options: ReadOptions,
  ) -> RowsIterator {
    unimplemented!()
  }
}

impl<'a> DatabaseColumnFamily<'a> for (&'a Database, &'a ColumnFamily) {
  fn get_pinned(&self, key: &[u8]) -> Result<Option<PinnableSlice>> {
    Ok(self.0.get_pinned_cf(self.1, key)?)
  }

  fn get_pinned_opt<K>(
    &self,
    key: K,
    options: &ReadOptions,
  ) -> Result<Option<PinnableSlice>>
  where
    K: AsRef<[u8]>,
  {
    Ok(self.0.get_pinned_cf_opt(self.1, key, options)?)
  }

  fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
    Ok(self.0.put_cf(self.1, key, value)?)
  }

  fn batch_put(
    &self,
    batch: &mut WriteBatchWithTransaction<true>,
    key: &[u8],
    value: &[u8],
  ) {
    batch.put_cf(self.1, &key, value);
  }

  fn batch_delete(
    &self,
    batch: &mut WriteBatchWithTransaction<true>,
    key: &[u8],
  ) {
    batch.delete_cf(self.1, key);
  }

  fn iterator(&self, mode: IteratorMode) -> RowsIterator {
    self.0.iterator_cf(self.1, mode)
  }

  fn iterator_opt(
    &self,
    options: ReadOptions,
    mode: IteratorMode,
  ) -> RowsIterator {
    self.0.iterator_cf_opt(self.1, options, mode)
  }

  fn prefix_iterator(&self, prefix: &[u8]) -> RowsIterator {
    self.0.prefix_iterator_cf(self.1, prefix)
  }

  fn prefix_iterator_opt(
    &self,
    prefix: &[u8],
    options: ReadOptions,
  ) -> RowsIterator {
    self.0.iterator_cf_opt(
      self.1,
      options,
      IteratorMode::From(prefix.as_ref(), Direction::Forward),
    )
  }
}
