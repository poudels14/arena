use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use anyhow::{Context, Result};
use indexmap::IndexMap;
use runtime::deno::core::{
  op2, JsBuffer, OpState, Resource, ResourceId, ToJsBuffer,
};
use runtime::permissions;
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
  pub content: JsBuffer,
  #[serde(default)]
  pub blobs: IndexMap<String, Option<JsBuffer>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
  id: String,
  pub content_length: u32,
  pub embeddings_count: u32,
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Only set in ops that send content
  pub content: Option<ToJsBuffer>,
  pub metadata: Option<IndexMap<String, Value>>,
}

#[op2(async)]
#[smi]
pub async fn op_cloud_vectordb_open(
  state: Rc<RefCell<OpState>>,
  #[string] path_str: String,
) -> Result<ResourceId> {
  let mut state = state.borrow_mut();
  let path = Path::new(&path_str);

  // Check access to db file
  // Check write access since `VectorDatabase` will create a new db if it
  // doesn't already exist
  permissions::resolve_write_path(&mut state, path)?;
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

#[op2(async)]
pub async fn op_cloud_vectordb_execute_query(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
  #[string] sql: String,
) -> Result<()> {
  let resource = get_db_resource(state, rid)?;
  let mut db = resource.db.borrow_mut();
  let mut client = sql::Client::new(&mut db);
  client.execute(&sql)?;

  Ok(())
}

#[op2(async)]
pub async fn op_cloud_vectordb_create_collection(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
  #[string] name: String,
  #[serde] config: query::Collection,
) -> Result<()> {
  let resource = get_db_resource(state, rid)?;
  resource
    .db
    .borrow_mut()
    .create_collection(name.as_str().into(), config)?;

  Ok(())
}

#[op2(async)]
#[serde]
pub async fn op_cloud_vectordb_list_collections(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
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

#[op2(async)]
#[serde]
pub async fn op_cloud_vectordb_get_collection(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
  #[string] id: String,
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

#[op2(async)]
pub async fn op_cloud_vectordb_add_document(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
  #[string] collection_id: String,
  #[string] doc_id: String,
  #[serde] document: NewDocument,
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

#[op2(async)]
#[serde]
pub async fn op_cloud_vectordb_list_documents(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
  #[string] collection_id: String,
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

#[op2(async)]
#[serde]
pub async fn op_cloud_vectordb_get_document(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
  #[string] collection_id: String,
  #[string] doc_id: String,
) -> Result<Option<Document>> {
  let resource = get_db_resource(state, rid)?;
  let document = resource
    .db
    .borrow()
    .get_document(collection_id.as_str().into(), doc_id.as_str().into())?;

  document
    .map(|doc| {
      Ok(Document {
        id: doc_id,
        embeddings_count: doc.embeddings_count,
        content_length: doc.content_length,
        content: Some(doc.content.into()),
        metadata: doc.metadata,
      })
    })
    .transpose()
}

#[op2(async)]
#[serde]
pub async fn op_cloud_vectordb_get_document_blobs(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
  #[string] collection_id: String,
  #[string] doc_id: String,
  #[serde] blob_keys: Vec<String>,
) -> Result<HashMap<String, ToJsBuffer>> {
  let resource = get_db_resource(state, rid)?;
  let blobs = resource.db.borrow().get_document_blobs(
    collection_id.as_str().into(),
    doc_id.as_str().into(),
    blob_keys.iter().map(|k| k.as_str().into()).collect(),
  )?;

  Ok(
    blobs
      .into_iter()
      .filter_map(|b| b.1.map(|c| (b.0, c.into())))
      .collect(),
  )
}

#[op2(async)]
pub async fn op_cloud_vectordb_set_document_embeddings(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
  #[string] collection_id: String,
  #[string] doc_id: String,
  #[serde] embeddings: Vec<query::Embeddings>,
) -> Result<()> {
  let resource = get_db_resource(state, rid)?;
  resource.db.borrow_mut().set_document_embeddings(
    collection_id.as_str().into(),
    doc_id.as_str().into(),
    embeddings,
  )?;
  Ok(())
}

#[op2(async)]
pub async fn op_cloud_vectordb_delete_document(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
  #[string] collection_id: String,
  #[string] doc_id: String,
) -> Result<()> {
  let resource = get_db_resource(state, rid)?;
  resource
    .db
    .borrow_mut()
    .delete_document(collection_id.as_str().into(), doc_id.as_str().into())?;
  Ok(())
}

#[op2(async)]
#[serde]
pub async fn op_cloud_vectordb_search_collection(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
  #[string] collection_id: String,
  #[serde] query: Vec<f32>,
  #[bigint] k: usize,
  #[serde] options: search::Options,
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
          let chunk: ToJsBuffer = doc.content[start..end].to_vec().into();

          let before_ctx = options.before_context.and_then(|size| {
            if start > 0 {
              Some(Into::<ToJsBuffer>::into(
                doc.content[0.max(start - size)..start].to_vec(),
              ))
            } else {
              None
            }
          });
          let after_ctx = options.after_context.map(|size| {
            Into::<ToJsBuffer>::into(
              doc.content[end..(end + size).min(doc.content.len())].to_vec(),
            )
          });

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

#[op2(async)]
pub async fn op_cloud_vectordb_compact_and_flush(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
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
