mod flatindex;

use super::VectorElement;
use anyhow::Result;

pub trait VectorIndex<'a, TopkResult> {
  fn topk(
    &'a self,
    query: &[VectorElement],
    k: usize,
  ) -> Result<Vec<(f32, &'a TopkResult)>>;
}
