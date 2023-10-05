use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use anyhow::{bail, Context, Result};
use common::deno::utils;
use deno_core::{
  op, OpState, Resource, ResourceId, StringOrBuffer, ZeroCopyBuf,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use vectordb::query::DocumentWithContent;
use vectordb::search::SearchOptions;
use vectordb::RowId;
use vectordb::{query, sql, DatabaseOptions, VectorDatabase};

mod search;

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
  pub metadata: Option<IndexMap<String, Value>>,
  pub content: StringOrBuffer,
  #[serde(default)]
  pub blobs: IndexMap<String, Option<StringOrBuffer>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
  id: String,
  pub content_length: u32,
  pub embeddings_count: u32,
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
  let collections = resource.db.borrow().list_collections()?;
  collections
    .into_iter()
    .map(|col| {
      Ok(Collection {
        id: col.id,
        documents_count: col.documents_count,
        dimension: col.dimension,
        metadata: col.metadata,
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
      metadata: document.metadata,
      content: document.content.to_vec(),
      blobs: document
        .blobs
        .iter()
        .filter_map(|(k, v)| {
          v.as_ref().map(|v| (k.as_str().into(), v.to_vec()))
        })
        .collect(),
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
  let documents = resource
    .db
    .borrow()
    .list_documents(collection_id.as_str().into())?;

  let documents = documents
    .into_iter()
    .map(|doc| {
      Ok(Document {
        id: doc.id,
        content_length: doc.content_length,
        embeddings_count: doc.embeddings_count,
        content: None,
        metadata: doc.metadata,
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
      let content = encoded_buffer(&doc.content, &content_encoding)?;
      Ok(Document {
        id: doc_id,
        embeddings_count: doc.embeddings_count,
        content_length: doc.content_length,
        content: Some(content),
        metadata: doc.metadata,
      })
    })
    .transpose()
}

#[op]
async fn op_cloud_vectordb_get_document_blobs(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  collection_id: String,
  doc_id: String,
  blob_keys: Vec<String>,
  encoding: Option<String>,
) -> Result<HashMap<String, StringOrBuffer>> {
  let resource = get_db_resource(state, rid)?;
  let blobs = resource.db.borrow().get_document_blobs(
    collection_id.as_str().into(),
    doc_id.as_str().into(),
    blob_keys.iter().map(|k| k.as_str().into()).collect(),
  )?;

  blobs
    .into_iter()
    .filter_map(|b| {
      b.1.map(|c| match encoding.as_ref() {
        Some(e) if e == "base-64" => Ok((b.0, encoded_buffer(&c, &encoding)?)),
        None => Ok((b.0, encoded_buffer(&c, &encoding)?)),
        _ => bail!("Only base-64 encoding is supported"),
      })
    })
    .collect()
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

#[op]
async fn op_cloud_vectordb_delete_document(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  collection_id: String,
  doc_id: String,
) -> Result<()> {
  let resource = get_db_resource(state, rid)?;
  resource
    .db
    .borrow_mut()
    .delete_document(collection_id.as_str().into(), doc_id.as_str().into())?;
  Ok(())
}

#[op]
async fn op_cloud_vectordb_search_collection(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  collection_id: String,
  query: Vec<f32>,
  k: usize,
  options: search::Options,
) -> Result<search::Result> {
  let resource = get_db_resource(state, rid)?;
  let db = resource.db.borrow();
  let searcher = vectordb::search::FsSearch::using(&db);
  let (results, metrics) = searcher.top_k(
    collection_id.as_str().into(),
    &query,
    k,
    SearchOptions {
      min_score: options.min_score,
    },
  )?;

  let mut documents: IndexMap<RowId, DocumentWithContent> = IndexMap::new();

  let embeddings = results
    .into_iter()
    .map(|result| {
      let document_row_id = result.row_id;
      let start = result.embedding_start as usize;
      let end = result.embedding_end as usize;

      if documents.get(&document_row_id).is_none() {
        let doc = db
          .get_document_by_row_id(&document_row_id)?
          .context("Document in search result not found")?;
        documents.insert(document_row_id.clone(), doc);
      };
      let doc = documents.get(&document_row_id).unwrap();

      let (content, context) = match options.include_chunk_content {
        true => {
          let chunk = encoded_buffer(
            &doc.content[start..end],
            &options.content_encoding,
          )?;

          let before_ctx = options
            .before_context
            .and_then(|size| {
              if start > 0 {
                Some(encoded_buffer(
                  &doc.content[0.max(start - size)..start],
                  &options.content_encoding,
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
                &doc.content[end..(end + size).min(doc.content.len())],
                &options.content_encoding,
              )
            })
            .transpose()?;

          let ctx = if before_ctx.is_none() && before_ctx.is_none() {
            None
          } else {
            Some((before_ctx, after_ctx))
          };
          (Some(chunk), ctx)
        }
        false => (None, None),
      };

      Ok(search::Embedding {
        score: result.score,
        document_id: doc.id.clone(),
        index: result.index as usize,
        start,
        end,
        content,
        context,
        metadata: result.metadata,
      })
    })
    .collect::<Result<Vec<search::Embedding>>>()?;

  Ok(search::Result {
    documents: documents
      .into_iter()
      .map(|(_, doc)| search::Document {
        id: doc.id,
        metadata: doc.metadata,
      })
      .collect(),
    embeddings,
    metrics,
  })
}

#[op]
async fn op_cloud_vectordb_compact_and_flush(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
) -> Result<()> {
  let resource = get_db_resource(state, rid)?;
  resource.db.borrow().compact_and_flush()?;
  Ok(())
}

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
  content: &[u8],
  encoding: &Option<String>,
) -> Result<StringOrBuffer> {
  match encoding.as_ref() {
    Some(e) if e == "utf-8" => Ok(StringOrBuffer::String(
      simdutf8::basic::from_utf8(&content)
        .context("decoding content to utf-8")?
        .to_owned(),
    )),
    Some(e) if e == "base-64" => {
      Ok(StringOrBuffer::String(base64::encode(content)))
    }
    _ => Ok(StringOrBuffer::Buffer(ZeroCopyBuf::ToV8(Some(
      content.to_vec().into_boxed_slice(),
    )))),
  }
}
