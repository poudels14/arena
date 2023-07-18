use super::collections::Collection;
use crate::db::rocks::cf::{column_handle, DatabaseColumnFamily};
use crate::db::rocks::PinnableSlice;
use crate::utils::bytes::ToBeBytes;
use anyhow::Result;
use bstr::BStr;
use rocksdb::{ColumnFamily, DBCompressionType, Options, ReadOptions, DB};

pub static DOC_CONTENTS_CF: &'static str = "document-contents";

pub fn cf<'a>(db: &'a DB) -> Result<impl DatabaseColumnFamily> {
  Ok((db, column_handle(db, DOC_CONTENTS_CF)?))
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

#[allow(dead_code)]
pub struct DocumentContentsHandle<'d> {
  collection_index: i32,
  handle: (&'d DB, &'d ColumnFamily),
}

#[allow(dead_code)]
impl<'d> DocumentContentsHandle<'d> {
  pub fn new(db: &'d DB, collection: &Collection) -> Result<Self> {
    Ok(Self {
      collection_index: collection.index,
      handle: (db, column_handle(db, DOC_CONTENTS_CF)?),
    })
  }

  pub fn get_pinned_slice(
    &self,
    doc_id: &BStr,
  ) -> Result<Option<PinnableSlice>> {
    let storage_doc_id = (self.collection_index, doc_id).to_be_bytes();

    let mut read_options = ReadOptions::default();
    read_options.fill_cache(false);
    self.handle.get_pinned_opt(storage_doc_id, &read_options)
  }
}
