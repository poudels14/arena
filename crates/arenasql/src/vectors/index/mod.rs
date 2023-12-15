pub mod flatindex;

use anyhow::Result;

use super::scoring::VectorElement;
use super::{Score, SimilarityScorer};

pub trait VectorIndex<'a, TopkResult> {
  fn topk(
    &'a self,
    scorer: &'a dyn SimilarityScorer,
    query: &[VectorElement],
    k: usize,
  ) -> Result<Vec<(Score, &'a TopkResult)>>;
}
