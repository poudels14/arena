use super::collections::Collection;
use super::{Database, Document};
use crate::db::rocks::cf::{column_handle, DatabaseColumnFamily};
use crate::utils::bytes::ToBeBytes;
use anyhow::bail;
use anyhow::Result;
use rkyv::{AlignedVec, Archive, Deserialize, Serialize};
use rocksdb::{
  ColumnFamily, DBCompressionType, Options, WriteBatchWithTransaction,
};

pub static DOC_EMBEDDINGS_CF: &'static str = "document-embeddings";

pub fn cf<'a>(db: &'a Database) -> Result<impl DatabaseColumnFamily> {
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

#[derive(Debug, Archive, Serialize, Deserialize)]
pub struct StoredEmbeddings {
  /// a list of i16 integers to use as a key prefix for the embeddings
  /// This allows us to return a list of embeddings and it's corresponding
  /// document slice with same prefix as the matched embedding.
  /// For example, if there are two paragraphs under a heading <h1>, then
  /// it might be better to return the text of the heading when one of
  /// the paragraphs is matched with the query. In order to do this, the
  /// heading should have a prefix of `[0,-1,...]` and the embeddings will
  /// have prefix of `[0,0,-1,...]` and `[0,1,-1,...]` where -1 is used as
  /// prefix terminator. So, if an embedding with prefix `[0,1,4,-1]` prefix
  /// is matched, we return chunks with prefix
  ///   - `[0,-1,{start},{end}]`
  ///   - `[0,1,-1,{start},{end}]`
  ///   - `[0,1,4,-1,{start},{end}]`
  /// All these chunks are "ancestors" of the matched embedding.
  ///
  // pub prefix: Vec<i16>,
  /// start index of the chunk
  pub start: u32,
  /// end index of the chunk
  pub end: u32,
  pub vectors: Vec<f32>,
  // Note(sagar): store as serialized bytes (Vec<u8>) since rkyv can't
  // serialize serde_json::Value
  pub metadata: Vec<u8>,
}

impl StoredEmbeddings {
  pub fn decode_unsafe<'a>(
    bytes: &'a mut [u8],
  ) -> &'a ArchivedStoredEmbeddings {
    unsafe { rkyv::archived_root::<StoredEmbeddings>(bytes) }
  }

  pub fn encode(embedding: &StoredEmbeddings) -> Result<AlignedVec> {
    let bytes = rkyv::to_bytes::<_, 1800>(embedding)?;
    Ok(bytes)
  }
}

#[allow(dead_code)]
pub struct DocumentEmbeddingsHandle<'d> {
  collection: &'d Collection,
  document: &'d Document,
  handle: (&'d Database, &'d ColumnFamily),
}

#[allow(dead_code)]
impl<'d> DocumentEmbeddingsHandle<'d> {
  pub fn new(
    db: &'d Database,
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
    batch: &mut WriteBatchWithTransaction<true>,
    index: u32,
    embedding: &StoredEmbeddings,
  ) -> Result<()> {
    if embedding.vectors.len() as u16 != self.collection.dimension {
      bail!(
        "Chunk embedding length doesn't match with collection dimension of {}",
        self.collection.dimension
      );
    }

    let chunk_id =
      (self.collection.index, self.document.index, index).to_be_bytes();
    let encoded_embeddings = StoredEmbeddings::encode(embedding)?;
    self.handle.batch_put(batch, &chunk_id, &encoded_embeddings);
    Ok(())
  }

  pub fn batch_delete(
    &self,
    batch: &mut WriteBatchWithTransaction<true>,
    index: u32,
  ) {
    let chunk_id =
      (self.collection.index, self.document.index, index).to_be_bytes();
    self.handle.batch_delete(batch, &chunk_id);
  }
}
