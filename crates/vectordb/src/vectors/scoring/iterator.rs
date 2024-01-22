use super::sortedscore::SortedSimilarityScores;
use super::{SimilarityScorer, VectorElement};

pub trait TopkIterator<'a>:
  Iterator<Item = &'a [VectorElement]> + Sized
{
  fn topk<T>(
    mut self,
    scorer: &dyn SimilarityScorer,
    query: &[VectorElement],
    k: usize,
  ) -> SortedSimilarityScores<usize> {
    let mut scores = SortedSimilarityScores::new(k);
    let mut counter = 0;

    loop {
      match self.next() {
        Some(v) => {
          let score = scorer.similarity(query, &v);
          scores.push((score, counter));
          counter += 1;
        }
        None => {
          return scores;
        }
      }
    }
  }
}

impl<'a, I: Iterator<Item = &'a [VectorElement]>> TopkIterator<'a> for I {}
