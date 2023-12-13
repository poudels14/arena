use ordered_float::OrderedFloat;

use super::glam::GlamDotSimilarity;
use super::ultraviolet::UltravioletDotSimilarity;

pub type Score = OrderedFloat<f32>;

pub type VectorElement = f32;

/// When using `bfloat16` feature, the stored vectors are
/// represented as u16 and converted to f32 during similarity
/// scoring.
/// Converting u16 to f32 with unchecked transmutate is really
/// fast and creating `ultraviolet::f32x8` from u16 seems to be
/// 30% faster than using f32;
/// TODO: IDK why that's the case benchmark properly to verify
#[cfg(feature = "bfloat16")]
pub type VectorElement = u16;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum SimilarityType {
  /// dot product similarity
  Dot,

  /// cosine similarity
  Cosine,

  /// euclidean distance similarity
  L2,
}

pub trait SimilarityScorer {
  /// Calculate the similarity score of two vectors
  /// The length of the vectors must be multiple of 4
  fn similarity_score(&self, vector: &[VectorElement], query: &[f32]) -> Score;
}

pub struct SimilarityScorerFactory;

impl SimilarityScorerFactory {
  pub fn get_default<'a>(t: SimilarityType) -> &'a dyn SimilarityScorer {
    Self::untraviolet(t)
  }

  #[allow(unused)]
  pub fn glam<'a>(t: SimilarityType) -> &'a dyn SimilarityScorer {
    match t {
      SimilarityType::Dot => &GlamDotSimilarity {},
      _ => unimplemented!(),
    }
  }

  #[allow(unused)]
  pub fn untraviolet<'a>(t: SimilarityType) -> &'a dyn SimilarityScorer {
    match t {
      SimilarityType::Dot => &UltravioletDotSimilarity {},
      _ => unimplemented!(),
    }
  }
}
