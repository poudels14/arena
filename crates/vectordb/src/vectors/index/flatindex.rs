use super::VectorIndex;
use crate::vectors::scoring::TopkIterator;
use crate::vectors::scoring::{SimilarityScorerFactory, SimilarityType};
use crate::vectors::VectorElement;

#[allow(dead_code)]
pub struct FlatVectorIndex<Metadata> {
  vectors: Vec<Vec<VectorElement>>,
  metadata: Vec<Metadata>,
}

#[allow(dead_code)]
impl<Metadata> FlatVectorIndex<Metadata>
where
  Metadata: Clone,
{
  pub fn from_slice(vectors: &[(Vec<VectorElement>, Metadata)]) -> Self {
    // Note(sagar): my assumption is that having vector embeddings in
    // a sequencial memory is better for performance since all those
    // vector embeddings have to be accessed one after another for calculating
    // similarity score. So, split them into two vectors
    let (vectors, metadata): (Vec<Vec<VectorElement>>, Vec<Metadata>) = vectors
      .iter()
      .map(|(v, m)| (v.to_owned(), m.clone()))
      .unzip();
    Self { vectors, metadata }
  }
}

impl<'a, Metadata> VectorIndex<'a, Metadata> for FlatVectorIndex<Metadata> {
  fn topk(
    &'a self,
    query: &[VectorElement],
    k: usize,
  ) -> anyhow::Result<Vec<(f32, &'a Metadata)>> {
    let scorer = SimilarityScorerFactory::get_default(SimilarityType::Dot);
    let scores = self
      .vectors
      .iter()
      .map(|v| v.as_ref())
      .topk::<usize>(scorer, query, k);

    Ok(
      scores
        .as_vec()
        .iter()
        .map(|(score, idx)| (*score, &self.metadata[*idx]))
        .collect::<Vec<(f32, &'a Metadata)>>(),
    )
  }
}

#[cfg(test)]
mod tests {
  use crate::vectors::index::flatindex::FlatVectorIndex;
  use crate::vectors::index::VectorIndex;
  use anyhow::Result;

  #[test]
  fn test_vector_collection_query() -> Result<()> {
    let collection = FlatVectorIndex::from_slice(&vec![
      (vec![0.4, 0.3, 0.2, 0.1], b"second".to_vec()),
      (vec![0.1, 0.2, 0.3, 0.4], b"first".to_vec()),
    ]);

    let query = vec![1., 1., 2., 0.25];
    let topk = collection.topk(&query, 1)?;
    let topk: Vec<&str> = topk.iter().map(|(_, m)| m).collect();

    assert_eq!(topk, vec!["first"]);
    Ok(())
  }
}
