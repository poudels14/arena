use super::collections::Collection;
use super::Document;
use crate::db::rocks::cf::{column_handle, DatabaseColumnFamily};
use crate::utils::bytes::ToBeBytes;
use anyhow::bail;
use anyhow::Result;
use rocksdb::{
  ColumnFamily, DBCompressionType, Options, WriteBatchWithTransaction, DB,
};
use serde::{Deserialize, Serialize};

pub static DOC_EMBEDDINGS_CF: &'static str = "document-embeddings";

pub fn cf<'a>(db: &'a DB) -> Result<impl DatabaseColumnFamily> {
  Ok((db, column_handle(db, DOC_EMBEDDINGS_CF)?))
}

pub fn get_db_options() -> Options {
  let mut opt = Options::default();
  opt.set_compression_type(DBCompressionType::Lz4);

  // enable blob files
  opt.set_enable_blob_files(true);
  opt.set_enable_blob_gc(true);
  // this isn't neessary in WAL mode but set it anyways
  opt.set_atomic_flush(true);
  opt
}

#[derive(Debug, Serialize)]
pub struct EmbeddingsSlice<'a> {
  /// start index of the chunk
  pub start: u32,
  /// end index of the chunk
  pub end: u32,
  pub vectors: &'a [f32],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoredEmbeddings {
  /// start index of the chunk
  pub start: u32,
  /// end index of the chunk
  pub end: u32,
  pub vectors: Vec<f32>,
}

#[allow(dead_code)]
pub struct DocumentEmbeddingsHandle<'d> {
  collection: &'d Collection,
  document: &'d Document,
  handle: (&'d DB, &'d ColumnFamily),
}

#[allow(dead_code)]
impl<'d> DocumentEmbeddingsHandle<'d> {
  pub fn new(
    db: &'d DB,
    collection: &'d Collection,
    document: &'d Document,
  ) -> Result<Self> {
    Ok(Self {
      collection,
      document,
      handle: (db, column_handle(db, DOC_EMBEDDINGS_CF)?),
    })
  }

  pub fn batch_put(
    &self,
    batch: &mut WriteBatchWithTransaction<false>,
    index: u32,
    embedding: &EmbeddingsSlice,
  ) -> Result<()> {
    if embedding.vectors.len() as u16 != self.collection.dimension {
      bail!(
        "Chunk embedding length doesn't match with collection dimension of {}",
        self.collection.dimension
      );
    }

    let chunk_id =
      (self.collection.index, self.document.index, index).to_be_bytes();
    self
      .handle
      .batch_put(batch, &chunk_id, &rmp_serde::to_vec(embedding)?);
    Ok(())
  }
}
