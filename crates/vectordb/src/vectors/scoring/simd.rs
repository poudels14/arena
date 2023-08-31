use super::{Score, SimilarityScorer, VectorElement};
use std::ops::{AddAssign, MulAssign};
use wide::f32x8;

#[derive(Clone)]
pub struct SimdDotSimilarity;

impl SimilarityScorer for SimdDotSimilarity {
  fn similarity(
    &self,
    query: &[VectorElement],
    vector: &[VectorElement],
  ) -> Score {
    let mut score = f32x8::new([0., 0., 0., 0., 0., 0., 0., 0.]);
    query
      .chunks_exact(8)
      .map(f32x8::from)
      .zip(vector.chunks_exact(8).map(f32x8::from))
      .for_each(|(mut a, b)| {
        a.mul_assign(b);
        score.add_assign(a);
      });

    score.reduce_add()
  }
}

#[cfg(test)]
mod tests {
  use super::SimdDotSimilarity;
  use crate::vectors::scoring::SimilarityScorer;

  #[test]
  fn test_simd_scorer() {
    let v1: Vec<f32> = vec![0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.2, 0.2];
    let v2: Vec<f32> = vec![1., 1., 1., 1., 1., 1., 1., 1.];

    let scorer = SimdDotSimilarity {};
    let simd_dot = scorer.similarity(&v1, &v2);

    let simple_dot_product =
      v1.iter().zip(&v2).map(|(a, b)| a * b).sum::<f32>();

    // yolo comparing floats
    assert_eq!(simd_dot, 1.0);
    assert_eq!(simple_dot_product, 1.0);
  }
}
