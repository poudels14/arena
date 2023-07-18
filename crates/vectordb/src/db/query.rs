use bstr::BString;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Collection {
  pub dimension: u16,
  pub metadata: Option<IndexMap<String, Value>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Document {
  pub id: BString,
  pub content_length: u32,
  pub chunks: Vec<(u32, u32)>,
  pub metadata: Option<Vec<u8>>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct DocumentWithContent {
  pub content_length: u32,
  pub chunks_count: u32,
  pub content: Vec<u8>,
  pub metadata: Option<IndexMap<String, Value>>,
}

#[derive(Default, Deserialize)]
pub struct AddDocumentQuery {
  pub content: Vec<u8>,
  pub metadata: Option<IndexMap<String, Value>>,
}

#[derive(Debug)]
pub struct DocumentIndexOption {
  // pub centroids_ratio:
}

#[derive(Debug, Deserialize)]
pub struct Embeddings {
  pub start: u32,
  pub end: u32,
  pub vectors: Vec<f32>,
  pub metadata: Option<Vec<u8>>,
}

#[derive(Debug, Serialize)]
pub struct ChunkEmbedding {
  pub document_id: BString,
  pub start: u32,
  pub end: u32,
  pub vectors: Vec<f32>,
}
