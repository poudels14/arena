use super::VectorIndex;
use crate::vectors::scoring::{SortedSimilarityScores, VectorElement};
use crate::vectors::{Score, SimilarityScorer};

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
    scorer: &'a dyn SimilarityScorer,
    query: &[VectorElement],
    k: usize,
  ) -> anyhow::Result<Vec<(Score, &'a Metadata)>> {
    let mut scores = SortedSimilarityScores::<usize>::new(k);
    self
      .vectors
      .iter()
      .enumerate()
      .for_each(|(id, v)| scores.push((scorer.similarity_score(v, query), id)));

    Ok(
      scores
        .as_vec()
        .iter()
        .map(|score| (score.0, &self.metadata[score.1]))
        .collect::<Vec<(Score, &'a Metadata)>>(),
    )
  }
}

#[cfg(test)]
mod tests {
  use crate::vectors::index::flatindex::FlatVectorIndex;
  use crate::vectors::index::VectorIndex;
  use crate::vectors::{SimilarityScorerFactory, SimilarityType};
  use anyhow::Result;

  #[test]
  fn test_vector_collection_query() -> Result<()> {
    let collection = FlatVectorIndex::from_slice(&vec![
      (vec![0.4, 0.3, 0.2, 0.1], b"second".to_vec()),
      (vec![0.1, 0.2, 0.3, 0.4], b"first".to_vec()),
    ]);

    let scorer = SimilarityScorerFactory::get_default(SimilarityType::Dot);
    let query = vec![1., 1., 2., 0.25];
    let topk = collection.topk(scorer, &query, 1)?;
    let topk: Vec<&str> = topk
      .iter()
      .map(|(_, m)| std::str::from_utf8(m).unwrap())
      .collect();

    assert_eq!(topk, vec!["second"]);
    Ok(())
  }
}
