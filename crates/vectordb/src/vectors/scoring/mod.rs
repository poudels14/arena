mod glam;
pub mod sortedscore;

use self::glam::GlamDotSimilarity;
use super::VectorElement;

pub type Score = f32;

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
  fn similarity(
    &self,
    query: &[VectorElement],
    vector: &[VectorElement],
  ) -> Score;
}

pub struct SimilarityScorerFactory;

impl SimilarityScorerFactory {
  pub fn get_default<'a>(t: SimilarityType) -> &'a dyn SimilarityScorer {
    match t {
      SimilarityType::Dot => &GlamDotSimilarity {},
      _ => unimplemented!(),
    }
  }
}
