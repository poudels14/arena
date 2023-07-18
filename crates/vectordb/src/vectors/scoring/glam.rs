use super::{Score, SimilarityScorer, VectorElement};
use glam::Vec4;

#[derive(Clone)]
pub struct GlamDotSimilarity;

impl SimilarityScorer for GlamDotSimilarity {
  fn similarity(
    &self,
    query: &[VectorElement],
    vector: &[VectorElement],
  ) -> Score {
    // TODO(sagar): use loop of 4 iter and crunchy crate to make sure the loop
    // is unrolled. This should make it faster
    // TODO(sagar): using MulAssign might be faster
    query
      .chunks_exact(4)
      .map(Vec4::from_slice)
      .zip(vector.chunks_exact(4).map(Vec4::from_slice))
      .map(|(a, b)| a.dot(b))
      .sum::<f32>()
  }
}
