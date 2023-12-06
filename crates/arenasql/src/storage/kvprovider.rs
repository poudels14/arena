use strum_macros::{Display, EnumIter, EnumString};

use crate::Result;

pub trait RowIterator {
  fn key(&self) -> Option<&[u8]>;

  fn get(&self) -> Option<(&[u8], &[u8])>;

  fn next(&mut self);
}

/// Use different key value groups to store different type of data.
/// This makes it easier to handle different data type differently.
/// For example, we can avoid index data from being backed up since
/// index can be re-created from rows
#[derive(Copy, Clone, Debug, EnumString, Display, EnumIter)]
pub enum KeyValueGroup {
  /// Used to store locks and frequentyly updated keys like row_id counter
  #[strum(serialize = "LOCKS")]
  Locks = 0,
  /// All schemas (database, table, etc) are stored under this key type
  #[strum(serialize = "SCHEMAS")]
  Schemas = 1,
  /// Table indices are stored under this key space
  #[strum(serialize = "INDEXES")]
  Indexes = 2,
  /// Row data is stored under this key space
  #[strum(serialize = "ROWS")]
  Rows = 3,
}

/// This is the interface to write key/values to the database.
/// The implementation of this trait doesn't have to be thread
/// safe since the transaction manager ensures the thread safety.
pub trait KeyValueProvider {
  /// Update the value of the given key atomically.
  /// This should return error if the key was modified
  /// by another transaction after the value was read first by
  /// this transaction.
  fn atomic_update(
    &self,
    group: KeyValueGroup,
    key: &[u8],
    updater: &dyn Fn(Option<Vec<u8>>) -> Result<Vec<u8>>,
  ) -> Result<Vec<u8>>;

  fn get(&self, group: KeyValueGroup, key: &[u8]) -> Result<Option<Vec<u8>>>;

  fn get_for_update(
    &self,
    group: KeyValueGroup,
    key: &[u8],
    exclusive: bool,
  ) -> Result<Option<Vec<u8>>>;

  /// This scan will return values as long as prefix matches
  /// If prefix doesn't match, iterator is done and it returns None
  fn scan_with_prefix(
    &self,
    _group: KeyValueGroup,
    _prefix: &[u8],
  ) -> Result<Box<dyn RowIterator>> {
    unimplemented!()
  }

  fn put(&self, group: KeyValueGroup, key: &[u8], value: &[u8]) -> Result<()> {
    self.put_all(group, &vec![(key, value)])
  }

  fn put_all(
    &self,
    group: KeyValueGroup,
    rows: &[(&[u8], &[u8])],
  ) -> Result<()>;

  fn commit(&self) -> Result<()>;

  fn rollback(&self) -> Result<()>;
}
