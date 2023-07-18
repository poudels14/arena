use crate::db::rocks::cf::RowsIterator;
use crate::db::rocks::cf::{column_handle, DatabaseColumnFamily};
use anyhow::Context;
use anyhow::Result;
use bstr::BStr;
use indexmap::IndexMap;
use rocksdb::IteratorMode;
use rocksdb::{ColumnFamily, WriteBatchWithTransaction, DB};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub static COLLECTIONS_CF: &'static str = "collections";

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
  /// The index is used to prefix all the documents in the collection.
  /// Since we need a fixed length prefix for all documents, use i32
  /// number as an index.
  /// 32 bit index is kind of an overkill tbh
  pub index: i32,
  /// The total number of documents in this collection
  pub documents_count: u32,
  pub dimension: u16,
  pub metadata: Option<IndexMap<String, Value>>,
}

pub fn cf(db: &DB) -> Result<impl DatabaseColumnFamily> {
  Ok((db, column_handle(&db, COLLECTIONS_CF)?))
}

pub struct CollectionsHandle<'d> {
  handle: (&'d DB, &'d ColumnFamily),
}

impl<'d> CollectionsHandle<'d> {
  pub fn new(db: &'d DB) -> Result<Self> {
    Ok(Self {
      handle: (db, column_handle(db, COLLECTIONS_CF)?),
    })
  }

  /// Get a document of the given id
  /// The id is document id without the collection index prefix
  pub fn get(&self, id: &BStr) -> Result<Option<Collection>> {
    let collection = self.handle.get_pinned(&id)?;

    match collection {
      Some(collection) => rmp_serde::from_slice(&collection)
        .context("Failed to deserialize collection"),
      None => Ok(None),
    }
  }

  pub fn put(&self, id: &BStr, collection: &Collection) -> Result<()> {
    self.handle.put(&id, &rmp_serde::to_vec(&collection)?)
  }

  pub fn iterator(&self, mode: IteratorMode) -> RowsIterator {
    self.handle.0.iterator_cf(self.handle.1, mode)
  }

  pub fn batch_put(
    &self,
    batch: &mut WriteBatchWithTransaction<false>,
    id: &BStr,
    collection: &Collection,
  ) -> Result<()> {
    self
      .handle
      .batch_put(batch, &id, &rmp_serde::to_vec(&collection)?);
    Ok(())
  }
}
