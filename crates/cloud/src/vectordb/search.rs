use deno_core::StringOrBuffer;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use vectordb::search::SearchMetrics;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
  #[serde(default)]
  pub include_chunk_content: bool,
  #[serde(default)]
  pub content_encoding: Option<String>,
  /// if set, only the embeddings with score greater or equal to this score
  /// will be returned
  #[serde(default)]
  pub min_score: Option<f32>,
  /// number of bytes before the matched embeddings to include in the response
  #[serde(default)]
  pub before_context: Option<usize>,
  /// number of bytes after the matched embeddings to include in the response
  #[serde(default)]
  pub after_context: Option<usize>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
  pub metrics: SearchMetrics,
  pub documents: Vec<Document>,
  pub embeddings: Vec<Embedding>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
  /// Document id
  pub id: String,
  /// Document metadata
  pub metadata: Option<IndexMap<String, Value>>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Embedding {
  pub score: f32,
  pub document_id: String,
  pub index: usize,
  pub start: usize,
  pub end: usize,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub content: Option<StringOrBuffer>,

  /// Only set if before/after_context is non-zero
  #[serde(skip_serializing_if = "Option::is_none")]
  pub context: Option<(Option<StringOrBuffer>, Option<StringOrBuffer>)>,
  /// Chunk/embedding metadata
  pub metadata: IndexMap<String, Value>,
}
