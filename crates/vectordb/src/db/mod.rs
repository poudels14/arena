mod errors;
pub mod query;
pub(crate) mod rocks;
pub(crate) mod storage;
use self::storage::collections::COLLECTIONS_CF;
use self::storage::contents::DOC_CONTENTS_CF;
use self::storage::documents::DOCUMENTS_CF;
use self::storage::embeddings::{StoredEmbeddings, DOC_EMBEDDINGS_CF};
use self::storage::{
  Collection, CollectionsHandle, Database, Document, DocumentBlobsHandle,
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
  ColumnFamilyDescriptor, DBCompressionType, FlushOptions, IteratorMode,
  OptimisticTransactionDB, Options, ReadOptions,
};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

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
  pub(crate) db: Database,
  doc_read_options: ReadOptions,
  pub(crate) collections_cache:
    Arc<RwLock<IndexMap<BString, Arc<RwLock<storage::Collection>>>>>,
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

    let db: Database = OptimisticTransactionDB::open_cf_descriptors(
      &opts,
      path,
      vec![
        ColumnFamilyDescriptor::new(COLLECTIONS_CF, Options::default()),
        ColumnFamilyDescriptor::new(COLLECTIONS_CF, Options::default()),
        ColumnFamilyDescriptor::new(DOCUMENTS_CF, Options::default()),
        ColumnFamilyDescriptor::new(
          DOC_CONTENTS_CF,
          storage::contents::get_db_options(),
        ),
        ColumnFamilyDescriptor::new(
          DOC_EMBEDDINGS_CF,
          storage::embeddings::get_db_options(),
        ),
      ],
    )?;

    let mut doc_read_options = ReadOptions::default();
    doc_read_options.fill_cache(false);
    Ok(Self {
      path: path.to_string(),
      opts,
      db,
      doc_read_options,
      collections_cache: Arc::new(RwLock::new(IndexMap::new())),
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
    // and use transaction
    let collection_count: u32 = collections_h
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
      next_doc_index: 0,
      dimension: col.dimension,
      metadata: col.metadata,
      blobs: HashSet::new(),
    };
    collections_h.put(id, &stored_collection)?;

    self
      .collections_cache
      .write()
      .map_err(lock_error)?
      .insert(id.to_owned(), Arc::new(RwLock::new(stored_collection)));
    Ok(())
  }

  /// Get collection info by id
  #[allow(dead_code)]
  pub fn get_collection(&self, id: &BStr) -> Result<Option<Collection>> {
    let res = self.get_internal_collection(id);

    match res {
      Ok(col) => col.read().map(|c| Some(c.clone())).map_err(lock_error),
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
    doc: query::Document,
  ) -> Result<()> {
    let collection = self.get_internal_collection(collection_id)?;
    let mut collection = collection.write().map_err(lock_error)?;
    let documents_cf = storage::documents::cf(&self.db)?;
    if collection.next_doc_index >= u32::MAX {
      bail!(
        "A single collection can't have more than {} documents",
        u32::MAX
      );
    }

    // Note(sagar): prefix the doc id with 2 bytes (u16) collection index
    let storage_doc_id: Vec<u8> = (collection.index, doc_id).to_be_bytes();

    if documents_cf.get_pinned(&storage_doc_id)?.is_some() {
      bail!(
        "Document already exists. It must be deleted before adding updated doc"
      );
    }

    let document = storage::Document {
      index: collection.next_doc_index,
      content_length: doc.content.len() as u32,
      chunks_count: 0,
      metadata: doc.metadata,
    };

    let blobs_h = DocumentBlobsHandle::new(&self.db, &collection, &document)?;
    let txn = self.db.transaction();
    let mut batch = txn.get_writebatch();

    documents_cf.batch_put(
      &mut batch,
      &storage_doc_id,
      &rmp_serde::to_vec(&document)?,
    );
    blobs_h.batch_put_content(&mut batch, &doc.content);

    doc.blobs.iter().for_each(|(key, blob)| {
      blobs_h.batch_put_blob(&mut batch, key, &blob);
    });

    let collections_h = CollectionsHandle::new(&self.db)?;
    collection.documents_count += 1;
    collection.next_doc_index += 1;
    collection.blobs.extend(
      doc
        .blobs
        .iter()
        .map(|(key, _)| key.clone())
        .collect::<Vec<String>>(),
    );
    collections_h.batch_put(&mut batch, collection_id, &collection)?;

    self
      .db
      .write(batch)
      .map_err(|e| anyhow!("Failed to commit transaction: {}", e))?;

    txn.commit()?;
    Ok(())
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
    let collection = collection.read().map_err(lock_error)?;
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
    let collection = collection.read().map_err(lock_error)?;

    let documents_h = DocumentsHandle::new(&self.db, &collection)?;
    let mut document = documents_h
      .get(&doc_id)?
      .context(format!("Couldn't find document with id = {}", doc_id))?;
    if document.chunks_count > 0 {
      bail!("Document chunks already added. To replace, delete them first");
    }

    let embeddings_h =
      DocumentEmbeddingsHandle::new(&self.db, &collection, &document)?;
    let txn = self.db.transaction();
    let mut batch = txn.get_writebatch();
    embeddings
      .iter()
      .enumerate()
      .map(|(index, embedding)| {
        if embedding.end < embedding.start {
          bail!("Embedding end index must be greater than start index");
        }
        embeddings_h.batch_put(
          &mut batch,
          index as u32,
          &StoredEmbeddings {
            start: embedding.start,
            end: embedding.end,
            vectors: embedding.vectors.to_owned(),
            metadata: rmp_serde::to_vec(
              &embedding.metadata.as_ref().unwrap_or(&IndexMap::new()),
            )?,
          },
        )?;
        Ok(())
      })
      .collect::<Result<()>>()?;

    document.chunks_count = embeddings.len() as u32;
    documents_h.batch_put(&mut batch, &doc_id, &document)?;

    self
      .db
      .write(batch)
      .map_err(|e| anyhow!("Error writing chunks: {}", e))?;

    txn.commit()?;
    Ok(())
  }

  #[allow(dead_code)]
  pub fn get_document(
    &self,
    col_id: &BStr,
    doc_id: &BStr,
  ) -> Result<Option<query::DocumentWithContent>> {
    let collection = self.get_internal_collection(col_id)?;
    let collection = collection.read().map_err(lock_error)?;
    let documents_h = DocumentsHandle::new(&self.db, &collection)?;
    let document = documents_h.get(doc_id)?;
    if let Some(doc) = document {
      let blob_h = DocumentBlobsHandle::new(&self.db, &collection, &doc)?;
      let content = blob_h
        .get_content()?
        .ok_or(anyhow!("Document content not found"))?
        .to_vec();

      return Ok(Some(query::DocumentWithContent {
        content_length: doc.content_length,
        chunks_count: doc.chunks_count,
        metadata: doc.metadata,
        content,
      }));
    }

    Ok(None)
  }

  #[allow(dead_code)]
  pub fn get_document_blobs(
    &self,
    col_id: &BStr,
    doc_id: &BStr,
    keys: Vec<String>,
  ) -> Result<Vec<(String, Option<Vec<u8>>)>> {
    let collection = self.get_internal_collection(col_id)?;
    let collection = collection.read().map_err(lock_error)?;
    let documents_h = DocumentsHandle::new(&self.db, &collection)?;
    let document = documents_h
      .get(doc_id)?
      .ok_or(anyhow!("Document not found"))?;
    let blob_h = DocumentBlobsHandle::new(&self.db, &collection, &document)?;
    keys
      .iter()
      .map(|key| {
        Ok((key.to_owned(), blob_h.get_blob(key)?.map(|b| b.to_vec())))
      })
      .collect()
  }

  /// Returns all the embeddings in a collection
  /// Warning: Only to be used for debug/test
  #[allow(dead_code)]
  pub fn scan_embeddings(
    &self,
    collection_id: &BStr,
  ) -> Result<Vec<query::ChunkEmbedding>> {
    let collection = self.get_internal_collection(collection_id)?;
    let collection = collection.read().map_err(lock_error)?;

    let mut document_id_by_index: Vec<BString> =
      vec![b"".into(); collection.next_doc_index as usize];
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
  pub fn delete_document(
    &mut self,
    collection_id: &BStr,
    doc_id: &BStr,
  ) -> Result<()> {
    let collection = self.get_internal_collection(collection_id)?;
    let mut collection = collection.write().map_err(lock_error)?;

    let collections_h = CollectionsHandle::new(&self.db)?;
    let documents_h = DocumentsHandle::new(&self.db, &collection)?;
    let document = documents_h
      .get(doc_id)?
      .ok_or(anyhow!("Document not found"))?;

    let txn = self.db.transaction();
    let mut batch = txn.get_writebatch();

    collection.documents_count -= 1;
    collections_h.batch_put(&mut batch, collection_id, &collection)?;

    let document_cf = storage::documents::cf(&self.db)?;
    let blob_h = DocumentBlobsHandle::new(&self.db, &collection, &document)?;

    let storage_doc_id: Vec<u8> = (collection.index, doc_id).to_be_bytes();
    document_cf.batch_delete(&mut batch, &storage_doc_id);
    blob_h.batch_delete(&mut batch);

    let embeddings_h =
      DocumentEmbeddingsHandle::new(&self.db, &collection, &document)?;
    (0..document.chunks_count).for_each(|idx| {
      embeddings_h.batch_delete(&mut batch, idx as u32);
    });

    self.db.write(batch)?;
    txn.commit()?;
    Ok(())
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
    Database::destroy(&Options::default(), path).map_err(|e| {
      anyhow!("{}. Make sure all database instances are closed.", e)
    })
  }

  /***************************************************************************/
  /***************************************************************************/
  /***************************************************************************/

  pub(crate) fn get_internal_collection(
    &'d self,
    id: &BStr,
  ) -> Result<Arc<RwLock<storage::Collection>>, errors::Error> {
    let cache = self.collections_cache.read().map_err(lock_error)?;
    let collection = cache.get(id).map(|c| c.clone());

    match collection {
      Some(col) => Ok(col),
      None => {
        let collections_h = CollectionsHandle::new(&self.db)?;
        let stored_collection = Arc::new(RwLock::new(
          collections_h
            .get(id)?
            .ok_or(Error::NotFound("Collection"))?,
        ));
        let mut cache = self.collections_cache.write().map_err(lock_error)?;
        cache.insert(id.to_owned(), stored_collection.clone());
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
