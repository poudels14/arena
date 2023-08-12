use anyhow::{bail, Context, Result};
use bstr::ByteSlice;
use common::deno::extensions::BuiltinExtension;
use common::deno::utils;
use deno_core::{
  op, Extension, OpState, Resource, ResourceId, StringOrBuffer, ZeroCopyBuf,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use vectordb::query::DocumentWithContent;
use vectordb::search::SearchOptions;
use vectordb::{query, sql, DatabaseOptions, VectorDatabase};

pub fn extension() -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(self::init()),
    runtime_modules: vec![],
    snapshot_modules: vec![],
  }
}

pub(crate) fn init() -> Extension {
  Extension::builder("arena/cloud/vectordb")
    .ops(vec![
      op_cloud_vectordb_open::decl(),
      op_cloud_vectordb_execute_query::decl(),
      op_cloud_vectordb_create_collection::decl(),
      op_cloud_vectordb_list_collections::decl(),
      op_cloud_vectordb_get_collection::decl(),
      op_cloud_vectordb_add_document::decl(),
      op_cloud_vectordb_list_documents::decl(),
      op_cloud_vectordb_get_document::decl(),
      op_cloud_vectordb_set_document_embeddings::decl(),
      op_cloud_vectordb_search_collection::decl(),
    ])
    .force_op_registration()
    .build()
}

#[allow(dead_code)]
struct VectorDatabaseResource {
  path: String,
  db: Rc<RefCell<VectorDatabase>>,
}

impl Resource for VectorDatabaseResource {
  fn close(self: Rc<Self>) {
    drop(self)
  }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
  id: String,
  pub documents_count: u32,
  pub dimension: u16,
  pub metadata: Option<IndexMap<String, Value>>,
}

#[derive(Deserialize)]
pub struct NewDocument {
  pub content: StringOrBuffer,
  pub metadata: Option<IndexMap<String, Value>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
  id: String,
  pub content_length: u32,
  pub chunks_count: u32,
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Only set in ops that send content
  pub content: Option<StringOrBuffer>,
  pub metadata: Option<IndexMap<String, Value>>,
}

#[op]
async fn op_cloud_vectordb_open(
  state: Rc<RefCell<OpState>>,
  path_str: String,
) -> Result<ResourceId> {
  let mut state = state.borrow_mut();
  let path = Path::new(&path_str);

  // Check access to db file
  // Check write access since `VectorDatabase` will create a new db if it
  // doesn't already exist
  utils::fs::resolve_write_path(&mut state, path)?;
  path
    .parent()
    .map(|dir| {
      if !dir.exists() {
        std::fs::create_dir_all(dir)?;
      }
      Ok::<(), anyhow::Error>(())
    })
    .transpose()?;

  let db = Rc::new(RefCell::new(VectorDatabase::open(
    &path_str,
    DatabaseOptions {
      enable_statistics: true,
    },
  )?));

  Ok(state.resource_table.add(VectorDatabaseResource {
    path: path_str.to_string(),
    db,
  }))
}

#[op]
async fn op_cloud_vectordb_execute_query(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  sql: String,
) -> Result<()> {
  let resource = get_db_resource(state, rid)?;
  let mut db = resource.db.borrow_mut();
  let mut client = sql::Client::new(&mut db);
  client.execute(&sql)?;

  Ok(())
}

#[op]
async fn op_cloud_vectordb_create_collection(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  name: String,
  config: query::Collection,
) -> Result<()> {
  let resource = get_db_resource(state, rid)?;
  resource
    .db
    .borrow_mut()
    .create_collection(name.as_str().into(), config)?;

  Ok(())
}

#[op]
async fn op_cloud_vectordb_list_collections(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
) -> Result<Vec<Collection>> {
  let resource = get_db_resource(state, rid)?;
  let mut collections = resource.db.borrow().list_collections()?;
  collections
    .iter_mut()
    .map(|(id, col)| {
      Ok(Collection {
        id: std::str::from_utf8(id)
          .map(|c| c.to_owned())
          .context("collection id should be utf-8")?,
        documents_count: col.documents_count,
        dimension: col.dimension,
        metadata: col.metadata.take(),
      })
    })
    .collect::<Result<Vec<Collection>>>()
}

#[op]
async fn op_cloud_vectordb_get_collection(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  id: String,
) -> Result<Option<Collection>> {
  let resource = get_db_resource(state, rid)?;
  let collection = resource.db.borrow().get_collection(id.as_str().into())?;

  Ok(collection.map(|c| Collection {
    id,
    documents_count: c.documents_count,
    dimension: c.dimension,
    metadata: c.metadata,
  }))
}

#[op]
async fn op_cloud_vectordb_add_document(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  collection_id: String,
  doc_id: String,
  document: NewDocument,
) -> Result<()> {
  let resource = get_db_resource(state, rid)?;
  resource.db.borrow_mut().add_document(
    collection_id.as_str().into(),
    doc_id.as_str().into(),
    query::Document {
      content: document.content.as_bytes().to_vec().into(),
      metadata: document.metadata,
    },
  )?;
  Ok(())
}

#[op]
async fn op_cloud_vectordb_list_documents(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  collection_id: String,
) -> Result<Vec<Document>> {
  let resource = get_db_resource(state, rid)?;
  let mut documents = resource
    .db
    .borrow()
    .list_documents(collection_id.as_str().into())?;

  let documents = documents
    .iter_mut()
    .map(|(id, doc)| {
      Ok(Document {
        id: std::str::from_utf8(id)
          .map(|s| s.to_owned())
          .context("document id should be utf-8")?,
        content_length: doc.content_length,
        chunks_count: doc.chunks_count,
        content: None,
        metadata: doc.metadata.take(),
      })
    })
    .collect::<Result<Vec<Document>>>();
  documents
}

#[op]
async fn op_cloud_vectordb_get_document(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  collection_id: String,
  doc_id: String,
  content_encoding: Option<String>,
) -> Result<Option<Document>> {
  let resource = get_db_resource(state, rid)?;
  let document = resource
    .db
    .borrow()
    .get_document(collection_id.as_str().into(), doc_id.as_str().into())?;

  document
    .map(|doc| {
      let content =
        encoded_buffer(doc.content.into_boxed_slice(), content_encoding)?;
      Ok(Document {
        id: doc_id,
        chunks_count: doc.chunks_count,
        content_length: doc.content_length,
        content: Some(content),
        metadata: doc.metadata,
      })
    })
    .transpose()
}

#[op]
async fn op_cloud_vectordb_set_document_embeddings(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  collection_id: String,
  doc_id: String,
  embeddings: Vec<query::Embeddings>,
) -> Result<()> {
  let resource = get_db_resource(state, rid)?;
  resource.db.borrow_mut().set_document_embeddings(
    collection_id.as_str().into(),
    doc_id.as_str().into(),
    embeddings,
  )?;
  Ok(())
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchCollectionOptions {
  #[serde(default)]
  pub include_chunk_content: bool,
  #[serde(default)]
  pub content_encoding: Option<String>,
  /// if set, only the chunks with score greater or equal to this score
  /// will be returned
  #[serde(default)]
  pub min_score: Option<f32>,
  /// number of bytes before the matched chunks to include in the response
  #[serde(default)]
  pub before_context: Option<usize>,
  /// number of bytes after the matched chunks to include in the response
  #[serde(default)]
  pub after_context: Option<usize>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchCollectionResult {
  pub score: f32,
  pub document_id: String,
  pub chunk_index: usize,
  pub start: usize,
  pub end: usize,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub content: Option<StringOrBuffer>,
  /// Only set if before/after_context is non-zero
  pub context: (Option<StringOrBuffer>, Option<StringOrBuffer>),
}

#[op]
async fn op_cloud_vectordb_search_collection(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  collection_id: String,
  query: Vec<f32>,
  k: usize,
  options: SearchCollectionOptions,
) -> Result<Vec<SearchCollectionResult>> {
  let resource = get_db_resource(state, rid)?;
  let db = resource.db.borrow();
  let searcher = vectordb::search::FsSearch::using(&db);
  let result = searcher.top_k(
    collection_id.as_str().into(),
    &query,
    k,
    SearchOptions {
      min_score: options.min_score,
    },
  )?;

  let mut documents: IndexMap<String, DocumentWithContent> = IndexMap::new();

  result
    .iter()
    .map(|(score, m)| {
      let document_id = std::str::from_utf8(&m.0)
        .context("document name should be utf-8")
        .map(|s| s.to_owned())?;
      let chunk_index = m.1 as usize;
      let start = m.2 as usize;
      let end = m.3 as usize;

      let (content, context) = match options.include_chunk_content {
        true => {
          if documents.get(&document_id).is_none() {
            let doc = db.get_document(
              collection_id.as_str().into(),
              document_id.as_str().into(),
            )?;
            if doc.is_none() {
              bail!("Document in search result not found");
            }
            documents.insert(document_id.clone(), doc.unwrap());
          };
          let doc = documents.get(&document_id).unwrap();

          let chunk = encoded_buffer(
            doc.content[start..end].to_vec().into_boxed_slice(),
            options.content_encoding.clone(),
          )?;

          let before_ctx = options
            .before_context
            .and_then(|size| {
              if start > 0 {
                Some(encoded_buffer(
                  doc.content[0.max(start - size)..start]
                    .to_vec()
                    .into_boxed_slice(),
                  options.content_encoding.clone(),
                ))
              } else {
                None
              }
            })
            .transpose()?;
          let after_ctx = options
            .after_context
            .map(|size| {
              encoded_buffer(
                doc.content[end..(end + size).min(doc.content.len())]
                  .to_vec()
                  .into_boxed_slice(),
                options.content_encoding.clone(),
              )
            })
            .transpose()?;
          (Some(chunk), (before_ctx, after_ctx))
        }
        false => (None, (None, None)),
      };

      Ok(SearchCollectionResult {
        score: *score,
        document_id: std::str::from_utf8(&m.0)
          .context("document name should be utf-8")
          .map(|s| s.to_owned())?,
        chunk_index,
        start,
        end,
        content,
        context,
      })
    })
    .collect::<Result<Vec<SearchCollectionResult>>>()
}

// #[op]
// async fn op_cloud_vectordb_compact_and_flush(
//   state: Rc<RefCell<OpState>>,
//   rid: ResourceId,
// ) -> Result<ResourceId> {
//   Ok(0)
// }

// #[op]
// async fn op_cloud_vectordb_destroy(
//   state: Rc<RefCell<OpState>>,
//   rid: ResourceId,
// ) -> Result<ResourceId> {
//   Ok(0)
// }

fn get_db_resource(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
) -> Result<Rc<VectorDatabaseResource>> {
  Ok(
    state
      .borrow_mut()
      .resource_table
      .get::<VectorDatabaseResource>(rid)?,
  )
}

fn encoded_buffer(
  content: Box<[u8]>,
  encoding: Option<String>,
) -> Result<StringOrBuffer> {
  match encoding.unwrap_or_default().as_ref() {
    "utf-8" => Ok(StringOrBuffer::String(
      simdutf8::basic::from_utf8(&content)
        .context("decoding content to utf-8")?
        .to_owned(),
    )),
    _ => Ok(StringOrBuffer::Buffer(ZeroCopyBuf::ToV8(Some(content)))),
  }
}
