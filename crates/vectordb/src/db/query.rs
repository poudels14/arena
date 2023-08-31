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
pub struct DocumentWithContent {
  pub content_length: u32,
  pub chunks_count: u32,
  pub metadata: Option<IndexMap<String, Value>>,
  pub content: Vec<u8>,
}

#[derive(Default, Deserialize)]
pub struct Document {
  pub metadata: Option<IndexMap<String, Value>>,
  pub content: Vec<u8>,
  /// A pair of blob key and value
  /// This field can be used to store arbitary files corresponding to
  /// the document like raw file and html content
  #[serde(default)]
  pub blobs: Vec<(String, Vec<u8>)>,
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
  pub metadata: Option<IndexMap<String, Value>>,
}

#[derive(Debug, Serialize)]
pub struct ChunkEmbedding {
  pub document_id: BString,
  pub start: u32,
  pub end: u32,
  pub vectors: Vec<f32>,
}
