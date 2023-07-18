use super::collections::Collection;
use crate::db::rocks::cf::{column_handle, DatabaseColumnFamily};
use crate::utils::bytes::ToBeBytes;
use anyhow::{Context, Result};
use bstr::{BStr, BString};
use indexmap::IndexMap;
use rocksdb::{ColumnFamily, WriteBatchWithTransaction, DB};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub static DOCUMENTS_CF: &'static str = "documents";

pub fn cf(db: &DB) -> Result<impl DatabaseColumnFamily> {
  Ok((db, column_handle(db, DOCUMENTS_CF)?))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
  /// use i32 just in case we wanna use negative index for sth like marking
  /// deletion, etc
  /// It's very very unlikely that we will ever store more than 2 billion
  /// documents in a single collection. So, using i32 saves half the space
  /// as compared to i64
  pub index: i32,
  pub content_length: u32,
  pub chunks_count: u32,
  pub metadata: Option<IndexMap<String, Value>>,
}

pub struct DocumentsHandle<'d> {
  collection_index: i32,
  handle: (&'d DB, &'d ColumnFamily),
}

impl<'d> DocumentsHandle<'d> {
  pub fn new(db: &'d DB, collection: &Collection) -> Result<Self> {
    Ok(Self {
      collection_index: collection.index,
      handle: (db, column_handle(db, DOCUMENTS_CF)?),
    })
  }

  /// Get a document of the given id
  /// The id is document id without the collection index prefix
  pub fn get(&self, id: &BStr) -> Result<Option<Document>> {
    let internal_doc_id = (self.collection_index, id).to_be_bytes();
    let doc = self.handle.get_pinned(&internal_doc_id)?;

    match doc {
      Some(doc) => {
        rmp_serde::from_slice(&doc).context("Failed to deserialize document")
      }
      None => Ok(None),
    }
  }

  pub fn batch_put(
    &self,
    batch: &mut WriteBatchWithTransaction<false>,
    // without collection index prefix
    document_id: &BStr,
    document: &Document,
  ) -> Result<()> {
    let internal_doc_id = (self.collection_index, document_id).to_be_bytes();
    self.handle.batch_put(
      batch,
      &internal_doc_id,
      &rmp_serde::to_vec(&document)?,
    );
    Ok(())
  }

  pub fn iterator(
    &'d self,
  ) -> impl Iterator<Item = Result<(BString, Document), anyhow::Error>> + 'd {
    let iter = self
      .handle
      .prefix_iterator(&self.collection_index.to_be_bytes());

    iter.into_iter().map(|doc| {
      let doc = doc?;
      let stored_doc = rmp_serde::from_slice::<Document>(&doc.1)?;

      // offset by 4 bytes since it's prefixed with i32 collection_id
      let doc_id = &doc.0[4..];
      Ok::<(BString, Document), anyhow::Error>((doc_id.into(), stored_doc))
    })
  }
}
