use super::row::RowId;
use super::Database;
use crate::db::rocks::cf::{column_handle, DatabaseColumnFamily};
use crate::utils::bytes::ToBeBytes;
use anyhow::{Context, Result};
use indexmap::IndexMap;
use rocksdb::{ColumnFamily, ReadOptions, WriteBatchWithTransaction};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub static DOCUMENTS_CF: &'static str = "documents";
pub static DOCUMENTS_ID_INDEX_CF: &'static str = "document-ids";

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
  /// It's very very unlikely that we will ever store more than 4 billion
  /// documents in a single collection. So, using u32 saves half the space
  /// as compared to u64
  pub index: u32,
  pub id: String,
  pub content_length: u32,
  pub embeddings_count: u32,
  pub metadata: Option<IndexMap<String, Value>>,
}

pub struct DocumentsHandle<'d> {
  collection_index: u32,
  doc_id_handle: (&'d Database, &'d ColumnFamily),
  handle: (&'d Database, &'d ColumnFamily),
}

impl<'d> DocumentsHandle<'d> {
  pub fn new(db: &'d Database, collection_index: u32) -> Result<Self> {
    Ok(Self {
      collection_index,
      doc_id_handle: (db, column_handle(db, DOCUMENTS_ID_INDEX_CF)?),
      handle: (db, column_handle(db, DOCUMENTS_CF)?),
    })
  }

  /// Get a document of the given id
  /// The id is document id without the collection index prefix
  pub fn get_by_id(&self, id: &str) -> Result<Option<Document>> {
    self
      .doc_id_handle
      .get_pinned(&(self.collection_index, id).to_be_bytes())?
      .and_then(|row_id| self.get(&row_id).transpose())
      .transpose()
  }

  pub fn get_row(&self, row_id: &RowId) -> Result<Option<Document>> {
    self.get(&row_id.to_be_bytes())
  }

  fn get(&self, row_id: &[u8]) -> Result<Option<Document>> {
    let doc = self.handle.get_pinned(row_id)?;

    match doc {
      Some(doc) => {
        rmp_serde::from_slice(&doc).context("Failed to deserialize document")
      }
      None => Ok(None),
    }
  }

  pub fn batch_put(
    &self,
    batch: &mut WriteBatchWithTransaction<true>,
    document: &Document,
  ) -> Result<()> {
    let row_id = RowId {
      collection_index: self.collection_index,
      row_index: document.index,
    }
    .to_be_bytes();

    self.doc_id_handle.batch_put(
      batch,
      &(self.collection_index, &document.id).to_be_bytes(),
      &row_id,
    );

    self
      .handle
      .batch_put(batch, &row_id, &rmp_serde::to_vec(&document)?);
    Ok(())
  }

  pub fn iterator(
    &'d self,
  ) -> impl Iterator<Item = Result<Document, anyhow::Error>> + 'd {
    let mut read_options = ReadOptions::default();
    read_options
      .set_iterate_upper_bound((self.collection_index + 1).to_be_bytes());
    let iter = self
      .handle
      .prefix_iterator_opt(&self.collection_index.to_be_bytes(), read_options);

    iter.into_iter().map(|doc| {
      let doc = doc?;
      let stored_doc = rmp_serde::from_slice::<Document>(&doc.1)?;
      Ok::<Document, anyhow::Error>(stored_doc)
    })
  }

  pub fn batch_delete(
    &self,
    batch: &mut WriteBatchWithTransaction<true>,
    document: &Document,
  ) -> Result<()> {
    self.doc_id_handle.batch_delete(
      batch,
      &(self.collection_index, &document.id).to_be_bytes(),
    );

    self.handle.batch_delete(
      batch,
      &RowId {
        collection_index: self.collection_index,
        row_index: document.index,
      }
      .to_be_bytes(),
    );
    Ok(())
  }
}
