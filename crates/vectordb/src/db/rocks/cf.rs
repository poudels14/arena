use super::PinnableSlice;
use anyhow::{anyhow, Result};
use rocksdb::{
  ColumnFamily, DBIteratorWithThreadMode, IteratorMode, ReadOptions,
  WriteBatchWithTransaction, DB,
};

pub type RowsIterator<'a> = DBIteratorWithThreadMode<'a, DB>;

pub fn column_handle<'a>(db: &'a DB, name: &str) -> Result<&'a ColumnFamily> {
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
    batch: &mut WriteBatchWithTransaction<false>,
    key: &[u8],
    value: &[u8],
  );

  fn iterator(&self, mode: IteratorMode) -> RowsIterator;

  fn iterator_opt(&self, options: ReadOptions, mode: IteratorMode) -> RowsIterator;

  fn prefix_iterator(&self, prefix: &[u8]) -> RowsIterator;
}

impl<'a> DatabaseColumnFamily<'a> for (&'a DB, &'a ColumnFamily) {
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
    batch: &mut WriteBatchWithTransaction<false>,
    key: &[u8],
    value: &[u8],
  ) {
    batch.put_cf(self.1, &key, value);
  }

  fn iterator(&self, mode: IteratorMode) -> RowsIterator {
    self.0.iterator_cf(self.1, mode)
  }

  fn iterator_opt(&self, options: ReadOptions, mode: IteratorMode) -> RowsIterator {
    self.0.iterator_cf_opt(self.1, options, mode)
  }

  fn prefix_iterator(&self, prefix: &[u8]) -> RowsIterator {
    self.0.prefix_iterator_cf(self.1, prefix)
  }
}
