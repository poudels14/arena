use indexmap::IndexMap;
use serde::Serialize;
use serde_json::Value;

use crate::RowId;

mod fs;
pub use fs::FsSearch;

#[derive(Debug, Default)]
pub struct SearchOptions {
  /// if set, only the chunks with score greater or equal to this score
  /// should be returned
  pub min_score: Option<f32>,
}

#[derive(Debug)]
pub struct MatchedEmbedding {
  pub score: f32,
  pub row_id: RowId,
  pub chunk_index: u32,
  pub embedding_start: u32,
  pub embedding_end: u32,
  pub metadata: IndexMap<String, Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchMetrics {
  pub total_embeddings_scanned: usize,
}
