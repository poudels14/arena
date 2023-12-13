use std::ops::AddAssign;

use ordered_float::OrderedFloat;
use ultraviolet::{f32x4, f32x8};

use super::scoring::{Score, SimilarityScorer};

#[derive(Clone)]
pub struct UltravioletDotSimilarity;

impl UltravioletDotSimilarity {
  fn similarity_score_x4(&self, vector: &[f32], query: &[f32]) -> Score {
    OrderedFloat(
      query
        .chunks_exact(4)
        .map(|v| f32x4::new(v.try_into().unwrap()))
        .zip(
          vector
            .chunks_exact(4)
            .map(|v| f32x4::new(v.try_into().unwrap())),
        )
        .fold(f32x4::new([0.0, 0.0, 0.0, 0.0]), |mut agg, (a, b)| {
          agg.add_assign(a * b);
          agg
        })
        .reduce_add(),
    )
  }

  fn similarity_score_x8(&self, vector: &[f32], query: &[f32]) -> Score {
    OrderedFloat(
      query
        .chunks_exact(8)
        .map(|v| f32x8::new(v.try_into().unwrap()))
        .zip(
          vector
            .chunks_exact(8)
            .map(|v| f32x8::new(v.try_into().unwrap())),
        )
        .fold(
          f32x8::new([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
          |mut agg, (a, b)| {
            agg.add_assign(a * b);
            agg
          },
        )
        .reduce_add(),
    )
  }
}

impl SimilarityScorer for UltravioletDotSimilarity {
  // Note: Using add_assign is [10-20%] faster
  // So, make it the default one
  fn similarity_score(&self, vector: &[f32], query: &[f32]) -> Score {
    if query.len() % 8 == 0 {
      self.similarity_score_x8(vector, query)
    } else {
      self.similarity_score_x4(vector, query)
    }
  }
}
