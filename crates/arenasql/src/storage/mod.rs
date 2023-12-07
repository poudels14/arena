mod kvprovider;
mod memory;
mod operators;
mod provider;
mod serializer;
mod transaction;

pub mod rocks;

pub use kvprovider::{KeyValueGroup, KeyValueProvider, RowIterator};
pub use memory::MemoryStorageProvider;
pub use operators::StorageOperator;
pub use provider::StorageProvider;
pub use serializer::*;
pub use transaction::Transaction;

#[macro_export]
macro_rules! last_table_id_key {
  () => {
    "m_last_table_id".as_bytes()
  };
}

#[macro_export]
macro_rules! last_table_index_id_key {
  () => {
    "m_last_table_index_id".as_bytes()
  };
}

#[macro_export]
macro_rules! last_row_id_of_table_key {
  ($table_id:expr) => {
    format!("m_t{}_last_rowid", $table_id).into_bytes()
  };
}

#[macro_export]
macro_rules! table_schema_key {
  ($catalog:expr, $schema:expr, $table:expr) => {
    format!("m_schema_c{}_s{}_t{}", $catalog, $schema, $table).as_bytes()
  };
}

#[macro_export]
macro_rules! index_rows_prefix_key {
  ($index_id:expr) => {
    vec!["i".as_bytes(), &$index_id.to_be_bytes(), "_".as_bytes()].concat()
  };
}

#[macro_export]
macro_rules! index_row_key {
  ($index_id:expr, $index_row:expr) => {
    vec![
      "i".as_bytes(),
      &$index_id.to_be_bytes(),
      "_".as_bytes(),
      $index_row,
    ]
    .concat()
  };
}

#[macro_export]
macro_rules! table_rows_prefix_key {
  ($table_id:expr) => {
    vec!["t".as_bytes(), &$table_id.to_be_bytes(), "_".as_bytes()].concat()
  };
}

#[macro_export]
macro_rules! table_row_key {
  ($table_id:expr, $row_id:expr) => {
    &vec![
      "t".as_bytes(),
      $table_id.to_be_bytes(),
      "_".as_bytes(),
      row_id,
    ]
    .concat()
  };
}
