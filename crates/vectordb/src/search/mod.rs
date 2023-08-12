mod fs;
pub use fs::FsSearch;

#[derive(Debug, Default)]
pub struct SearchOptions {
  /// if set, only the chunks with score greater or equal to this score
  /// should be returned
  pub min_score: Option<f32>,
}
