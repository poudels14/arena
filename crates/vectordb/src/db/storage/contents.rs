use super::{Database, Document};
use crate::db::rocks::cf::{column_handle, DatabaseColumnFamily};
use crate::db::rocks::PinnableSlice;
use crate::utils::bytes::ToBeBytes;
use anyhow::Result;
use rocksdb::{
  ColumnFamily, DBCompressionType, Options, ReadOptions,
  WriteBatchWithTransaction,
};

pub static DOC_CONTENTS_CF: &'static str = "document-contents";

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

#[allow(dead_code)]
pub struct DocumentBlobsHandle<'d> {
  collection_index: u32,
  document: &'d Document,
  handle: (&'d Database, &'d ColumnFamily),
}

#[allow(dead_code)]
impl<'d> DocumentBlobsHandle<'d> {
  pub fn new(
    db: &'d Database,
    collection_index: u32,
    document: &'d Document,
  ) -> Result<Self> {
    Ok(Self {
      collection_index,
      document,
      handle: (db, column_handle(db, DOC_CONTENTS_CF)?),
    })
  }

  pub fn batch_put_content(
    &self,
    batch: &mut WriteBatchWithTransaction<true>,
    content: &[u8],
  ) {
    self.batch_put_blob(batch, "content".into(), &content);
  }

  /// Put a blob with the given key corresponding to the document
  pub fn batch_put_blob(
    &self,
    batch: &mut WriteBatchWithTransaction<true>,
    blob_key: &str,
    content: &[u8],
  ) {
    self.handle.batch_put(
      batch,
      &(self.collection_index, self.document.index, "-", blob_key)
        .to_be_bytes(),
      &content,
    );
  }

  pub fn get_content(&self) -> Result<Option<PinnableSlice>> {
    self.get_blob("content".into())
  }

  pub fn get_blob(&self, blob_key: &str) -> Result<Option<PinnableSlice>> {
    let mut read_options = ReadOptions::default();
    read_options.fill_cache(false);
    self.handle.get_pinned_opt(
      (self.collection_index, self.document.index, "-", blob_key).to_be_bytes(),
      &read_options,
    )
  }

  /// Deletes all blobs including "content" for this collection/document
  pub fn batch_delete(
    &self,
    batch: &mut WriteBatchWithTransaction<true>,
    blob_key: &str,
  ) {
    self.handle.batch_delete(
      batch,
      &(self.collection_index, self.document.index, "-", blob_key)
        .to_be_bytes(),
    )
  }
}
