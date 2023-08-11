mod errors;
pub mod query;
pub(crate) mod rocks;
pub(crate) mod storage;
use self::storage::collections::COLLECTIONS_CF;
use self::storage::contents::DOC_CONTENTS_CF;
use self::storage::documents::DOCUMENTS_CF;
use self::storage::embeddings::{StoredEmbeddings, DOC_EMBEDDINGS_CF};
use self::storage::{
  Collection, CollectionsHandle, Document, DocumentContentsHandle,
  DocumentEmbeddingsHandle, DocumentsHandle,
};
use crate::db::rocks::cf::DatabaseColumnFamily;
use crate::utils::bytes::ToBeBytes;
use anyhow::{anyhow, bail, Context, Result};
use bitvec::field::BitField;
use bitvec::prelude::Msb0;
use bitvec::view::BitView;
use bstr::{BStr, BString, ByteSlice};
use errors::Error;
use indexmap::IndexMap;
use rocksdb::{
  DBCompressionType, FlushOptions, IteratorMode, Options, ReadOptions,
  WriteBatchWithTransaction, DB,
};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct DatabaseOptions {
  pub enable_statistics: bool,
}

/// Document vector database stores documents, their chunks and embeddings for
/// each chunks. The chunks are stored as indices of (begin, end) bytes of the
/// chunks. This prevents storing duplicate data for each document; first the
/// entire document and next for each chunks.
///
/// Documents and chunks can have metadata associated with them. When querying
/// the index, the metadata for matched chunk and documents are returned.
#[allow(dead_code)]
pub struct VectorDatabase {
  pub(crate) path: String,
  pub(crate) opts: Options,
  pub(crate) db: DB,
  doc_read_options: ReadOptions,
  pub(crate) collections_cache:
    Arc<Mutex<IndexMap<BString, Arc<Mutex<storage::Collection>>>>>,
}

impl<'d> VectorDatabase {
  #[allow(dead_code)]
  pub fn open(path: &str, options: DatabaseOptions) -> Result<Self> {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);
    opts.set_compression_type(DBCompressionType::Lz4);
    opts.set_max_background_jobs(0);

    // enable blob files
    opts.set_enable_blob_files(true);
    opts.set_enable_blob_gc(true);
    // this isn't neessary in WAL mode but set it anyways
    opts.set_atomic_flush(true);
    if options.enable_statistics {
      opts.enable_statistics();
    }

    let db = DB::open_cf_with_opts(
      &opts,
      path,
      vec![
        (COLLECTIONS_CF, Options::default()),
        (DOCUMENTS_CF, Options::default()),
        (DOC_CONTENTS_CF, storage::contents::get_db_options()),
        (DOC_EMBEDDINGS_CF, storage::embeddings::get_db_options()),
      ],
    )?;

    let mut doc_read_options = ReadOptions::default();
    doc_read_options.fill_cache(false);
    Ok(Self {
      path: path.to_string(),
      opts,
      db,
      doc_read_options,
      collections_cache: Arc::new(Mutex::new(IndexMap::new())),
    })
  }

  /// Create a vector collection
  #[allow(dead_code)]
  pub fn create_collection(
    &mut self,
    id: &BStr,
    col: query::Collection,
  ) -> Result<()> {
    if self.get_internal_collection(id).is_ok() {
      bail!("Collection with given id already exist");
    }

    let collections_h = CollectionsHandle::new(&self.db)?;
    // TODO(sagar): store collection counter on default column?
    let collection_count: i32 = collections_h
      .iterator(IteratorMode::Start)
      .count()
      .try_into()
      .map_err(|e| anyhow!("Error creating index: {}", e))?;

    if col.dimension % 4 != 0 {
      bail!("Dimension should be a multiple of 4");
    }

    let stored_collection = storage::Collection {
      index: collection_count,
      documents_count: 0,
      dimension: col.dimension,
      metadata: col.metadata,
    };
    collections_h.put(id, &stored_collection)?;

    self
      .collections_cache
      .lock()
      .map_err(lock_error)?
      .insert(id.to_owned(), Arc::new(Mutex::new(stored_collection)));
    Ok(())
  }

  /// Get collection info by id
  #[allow(dead_code)]
  pub fn get_collection(&self, id: &BStr) -> Result<Option<Collection>> {
    let res = self.get_internal_collection(id);

    match res {
      Ok(col) => col.lock().map(|c| Some(c.clone())).map_err(lock_error),
      Err(Error::NotFound(_)) => Ok(None),
      Err(e) => Err(e.into()),
    }
  }

  /// Lists all the collections
  /// This reads the collections from the database and don't use caching
  #[allow(dead_code)]
  pub fn list_collections(&self) -> Result<Vec<(BString, Collection)>> {
    let collections_cf = storage::collections::cf(&self.db)?;
    let iter = collections_cf.iterator(IteratorMode::Start);
    Ok(
      iter
        .map(|col| {
          let col = col?;
          let collection =
            rmp_serde::from_slice::<storage::Collection>(&col.1)?;
          Ok((col.0.as_ref().as_bstr().to_owned(), collection.clone()))
        })
        .collect::<Result<Vec<(BString, Collection)>>>()?,
    )
  }

  #[allow(dead_code)]
  pub fn delete_collection(&self) -> Result<()> {
    // TODO(sagar): delete all the documents/embeddings in the collection
    // and delete the collection
    bail!("not implemented");
  }

  /// Note(sagar): Since the chunks associated will be outdated when updating
  /// the doc, return error if the document already exists. To update,
  /// the document should be deleted first and then added.
  #[allow(dead_code)]
  pub fn add_document(
    &mut self,
    collection_id: &BStr,
    doc_id: &BStr,
    query: query::Document,
  ) -> Result<()> {
    let collection = self.get_internal_collection(collection_id)?;
    let mut collection = collection.lock().map_err(lock_error)?;
    let documents_cf = storage::documents::cf(&self.db)?;

    // Note(sagar): prefix the doc id with 2 bytes (u16) collection index
    let storage_doc_id: Vec<u8> = (collection.index, doc_id).to_be_bytes();

    if documents_cf.get_pinned(&storage_doc_id)?.is_some() {
      bail!(
        "Document already exists. It must be deleted before adding updated doc"
      );
    }

    let document = storage::Document {
      index: collection.documents_count as i32,
      content_length: query.content.len() as u32,
      chunks_count: 0,
      metadata: query.metadata,
    };

    let contents_cf = storage::contents::cf(&self.db)?;
    let mut batch = WriteBatchWithTransaction::<false>::default();
    documents_cf.batch_put(
      &mut batch,
      &storage_doc_id,
      &rmp_serde::to_vec(&document)?,
    );
    contents_cf.batch_put(&mut batch, &storage_doc_id, &query.content);

    let collections_h = CollectionsHandle::new(&self.db)?;
    collection.documents_count += 1;
    collections_h.batch_put(&mut batch, collection_id, &collection)?;

    self
      .db
      .write(batch)
      .map_err(|e| anyhow!("Failed to commit transaction: {}", e))
  }

  /// This will index the given document using ANN to make the search within
  /// the document faster. Use some sort of IVF index for now. Maybe just 3
  /// centroids with the document is good enough
  #[allow(dead_code)]
  pub fn index_document(
    &self,
    _doc_id: &[u8],
    _options: &query::DocumentIndexOption,
  ) -> Result<()> {
    unimplemented!("// TODO(sagar)");
  }

  /// Returns a map of (document_id => document)
  #[allow(dead_code)]
  pub fn list_documents(
    &self,
    col_id: &BStr,
  ) -> Result<Vec<(BString, Document)>> {
    let collection = self.get_internal_collection(col_id)?;
    let collection = collection.lock().map_err(lock_error)?;
    let documents_h = DocumentsHandle::new(&self.db, &collection)?;
    documents_h
      .iterator()
      .collect::<Result<Vec<(BString, Document)>>>()
  }

  // TODO(sagar): make it so that chunks can only be added one time
  // This is necessary to index the chunks
  /// Adds document chunks and their embeddings to the database
  #[allow(dead_code)]
  pub fn set_document_embeddings(
    &mut self,
    collection_id: &BStr,
    doc_id: &BStr,
    embeddings: Vec<query::Embeddings>,
  ) -> Result<()> {
    let collection = self.get_internal_collection(collection_id)?;
    let collection = collection.lock().map_err(lock_error)?;

    let documents_h = DocumentsHandle::new(&self.db, &collection)?;
    let document = documents_h
      .get(&doc_id)?
      .context(format!("Couldn't find document with id = {}", doc_id))?;
    if document.chunks_count > 0 {
      bail!("Document chunks already added. To replace, delete them first");
    }

    let embeddings_h =
      DocumentEmbeddingsHandle::new(&self.db, &collection, &document)?;
    let mut batch = WriteBatchWithTransaction::<false>::default();
    embeddings
      .iter()
      .enumerate()
      .map(|(index, embedding)| {
        if embedding.end < embedding.start {
          bail!("Embedding end index must be greater than start index");
        }
        embeddings_h.batch_put(
          &mut batch,
          index as i32,
          &StoredEmbeddings {
            start: embedding.start,
            end: embedding.end,
            vectors: embedding.vectors.to_owned(),
          },
        )?;
        Ok(())
      })
      .collect::<Result<()>>()?;
    documents_h.batch_put(&mut batch, &doc_id, &document)?;

    self
      .db
      .write(batch)
      .map_err(|e| anyhow!("Error writing chunks: {}", e))?;

    Ok(())
  }

  #[allow(dead_code)]
  pub fn get_document(
    &self,
    col_id: &BStr,
    doc_id: &BStr,
  ) -> Result<Option<query::DocumentWithContent>> {
    let collection = self.get_internal_collection(col_id)?;
    let collection = collection.lock().map_err(lock_error)?;
    let documents_h = DocumentsHandle::new(&self.db, &collection)?;
    let document = documents_h.get(doc_id)?;
    if let Some(doc) = document {
      let content_h = DocumentContentsHandle::new(&self.db, &collection)?;
      let content = content_h
        .get_pinned_slice(doc_id)?
        .ok_or(anyhow!("Document content not found"))?;
      return Ok(Some(query::DocumentWithContent {
        content_length: doc.content_length,
        chunks_count: doc.chunks_count,
        metadata: doc.metadata,
        content: content.to_vec(),
      }));
    }

    Ok(None)
  }

  /// Returns the db handle of the document content of given collection
  #[allow(dead_code)]
  pub fn get_document_content_handle(
    &'d self,
    collection_id: &BStr,
  ) -> Result<DocumentContentsHandle> {
    let collection = self.get_internal_collection(collection_id)?;
    let collection = collection.lock().map_err(lock_error)?;
    DocumentContentsHandle::new(&self.db, &collection)
  }

  /// Returns all the embeddings in a collection
  #[allow(dead_code)]
  pub fn scan_embeddings(
    &self,
    collection_id: &BStr,
  ) -> Result<Vec<query::ChunkEmbedding>> {
    let collection = self.get_internal_collection(collection_id)?;
    let collection = collection.lock().map_err(lock_error)?;

    let mut document_id_by_index: Vec<BString> =
      vec![b"".into(); collection.documents_count as usize];
    let document_h = DocumentsHandle::new(&self.db, &collection)?;

    document_h.iterator().for_each(|item| {
      if let Ok((id, doc)) = item {
        document_id_by_index[doc.index as usize] = id;
      }
    });

    let embeddings_cf = storage::embeddings::cf(&self.db)?;
    let mut read_options = ReadOptions::default();
    read_options.fill_cache(false);

    embeddings_cf
      .prefix_iterator(&collection.index.to_be_bytes())
      .map(|embedding| {
        let (key, mut embedding) = embedding?;
        let doc_index: usize = key[4..8].view_bits::<Msb0>().load_be();
        let embedding = StoredEmbeddings::decode_unsafe(&mut embedding);

        Ok(query::ChunkEmbedding {
          document_id: document_id_by_index[doc_index].clone(),
          start: embedding.start,
          end: embedding.end,
          // TODO(sagar): scanning takes ~15 times longer than dot product
          // My guess is, it's because of this vector clone here :(
          vectors: embedding.vectors.to_vec(),
        })
      })
      .collect()
  }

  #[allow(dead_code)]
  pub fn delete_document(&mut self, document_id: &[u8]) -> Result<()> {
    self.db.delete(document_id)?;
    Ok(())
  }

  #[allow(dead_code)]
  pub fn delete_chunks(&self, _doc_id: &BStr) -> Result<()> {
    unimplemented!()
  }

  #[allow(dead_code)]
  pub fn compact_and_flush(&self) -> Result<()> {
    // TODO(sagar)
    // IDK if this actually runs compaction
    self.db.compact_range(None::<&[u8]>, None::<&[u8]>);

    let mut flush_opt = FlushOptions::default();
    flush_opt.set_wait(true);
    self.db.flush()?;
    Ok(())
  }

  pub fn close(&mut self) -> Result<()> {
    self.db.cancel_all_background_work(true);
    Ok(())
  }

  #[allow(dead_code)]
  pub fn backup() -> Result<()> {
    // Note(sagar): look into flushing WAL/SST before creating a backup
    // TODO(sagar)
    bail!("Not implemented");
  }

  #[allow(dead_code)]
  pub fn destroy(path: &str) -> Result<()> {
    DB::destroy(&Options::default(), path).map_err(|e| {
      anyhow!("{}. Make sure all database instances are closed.", e)
    })
  }

  /***************************************************************************/
  /***************************************************************************/
  /***************************************************************************/

  pub(crate) fn get_internal_collection(
    &'d self,
    id: &BStr,
  ) -> Result<Arc<Mutex<storage::Collection>>, errors::Error> {
    let mut collections_cache =
      self.collections_cache.lock().map_err(lock_error)?;
    let collection = collections_cache.get(id).map(|c| c.clone());

    match collection {
      Some(col) => Ok(col),
      None => {
        let collections_h = CollectionsHandle::new(&self.db)?;
        let stored_collection = Arc::new(Mutex::new(
          collections_h
            .get(id)?
            .ok_or(Error::NotFound("Collection"))?,
        ));
        collections_cache.insert(id.to_owned(), stored_collection.clone());
        Ok(stored_collection)
      }
    }
  }
}

pub(crate) fn lock_error<E>(_: E) -> anyhow::Error {
  anyhow!("Error getting database lock")
}

impl<'d> Drop for VectorDatabase {
  fn drop(&mut self) {
    self.close().unwrap();
  }
}

#[allow(unused_imports)]
#[allow(dead_code)]
mod tests {
  use crate::db::{DatabaseOptions, VectorDatabase};
  use anyhow::Result;

  const DB_PATH: &'static str = "./test-vector-db";

  #[test]
  fn test_vector_rocks() -> Result<()> {
    let db = VectorDatabase::open(
      DB_PATH,
      DatabaseOptions {
        enable_statistics: true,
      },
    )?;

    drop(db);
    VectorDatabase::destroy(DB_PATH)?;
    Ok(())
  }
}
