use std::ops::AddAssign;

use glam::Vec4;
use ordered_float::OrderedFloat;

use super::scoring::{Score, SimilarityScorer};

#[derive(Clone)]
pub struct GlamDotSimilarity;

impl SimilarityScorer for GlamDotSimilarity {
  // Note: Using add_assign is [10-20%] faster
  // So, make it the default one
  fn similarity_score(&self, vector: &[f32], query: &[f32]) -> Score {
    OrderedFloat(
      query
        .chunks_exact(4)
        .map(Vec4::from_slice)
        .zip(vector.chunks_exact(4).map(Vec4::from_slice))
        .fold(
          Vec4::from_slice(&[0.0, 0.0, 0.0, 0.0]),
          |mut agg, (a, b)| {
            agg.add_assign(a * b);
            agg
          },
        )
        .to_array()
        .iter()
        .sum(),
    )
  }

  #[cfg(feature = "glam-dot")]
  fn similarity_score(&self, vector: &[f32], query: &[f32]) -> Score {
    OrderedFloat(
      query
        .chunks_exact(4)
        .map(Vec4::from_slice)
        .zip(vector.chunks_exact(4).map(Vec4::from_slice))
        .map(|(a, b)| a.dot(b))
        .sum::<f32>(),
    )
  }
}
