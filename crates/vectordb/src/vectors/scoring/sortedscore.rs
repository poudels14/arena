use derivative::Derivative;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

static FLOAT_MARGIN: f32 = 0.00000005;

#[derive(Derivative)]
#[derivative(PartialEq)]
struct ScoreWithIndex<T>(f32, #[derivative(PartialEq = "ignore")] T);

/// SortedSimilarityScores sorts and retains top-k scores and its index.
/// Since, higher similarity score mean the vectors are less similar, this
/// uses max-heap to store the top-k (index, score) and will remove highest
/// scores as new score is being inserted to maintain a list of most similar
/// vector index and their similarity score
pub struct SortedSimilarityScores<T> {
  k: usize,
  heap: BinaryHeap<ScoreWithIndex<Option<T>>>,
}

impl<T> SortedSimilarityScores<T> {
  pub fn new(k: usize) -> Self {
    Self {
      k,
      heap: BinaryHeap::with_capacity(2 * k + 2),
    }
  }

  pub fn push(&mut self, item: (f32, T)) {
    self.heap.push(ScoreWithIndex(item.0, Some(item.1)));
    if self.heap.len() > self.k + 1 {
      self.heap.pop();
    }
  }

  pub fn as_vec(self) -> Vec<(f32, T)> {
    self
      .heap
      .into_sorted_vec()
      .iter_mut()
      .take(self.k)
      // TODO(sagar): do I need option here?
      .map(move |item| (item.0, item.1.take().unwrap()))
      .collect::<Vec<(f32, T)>>()
  }
}

impl<T> Ord for ScoreWithIndex<T> {
  fn cmp(&self, other: &Self) -> Ordering {
    self.partial_cmp(other).unwrap_or(Ordering::Equal)
  }
}

impl<T> std::cmp::Eq for ScoreWithIndex<T> {}

impl<T> PartialOrd for ScoreWithIndex<T> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    let diff = self.0 - other.0;
    if diff < FLOAT_MARGIN {
      Some(Ordering::Less)
    } else if diff > FLOAT_MARGIN {
      Some(Ordering::Greater)
    } else {
      Some(Ordering::Equal)
    }
  }
}

#[cfg(test)]
mod sortedscore_tests {
  use super::SortedSimilarityScores;

  #[test]
  fn test_top_k() {
    let mut list = SortedSimilarityScores::new(2);
    list.push((5.0, 1));
    list.push((1.0, 2));
    list.push((6.0, 3));
    list.push((2.0, 4));

    assert_eq!(list.as_vec(), vec![(1., 2), (2., 4)]);
  }
}

#[cfg(test)]
mod reverseitem_tests {
  use super::ScoreWithIndex;

  #[test]
  fn test_score_with_smaller_score_smaller() {
    let item1 = ScoreWithIndex(1.0, 1);
    let item2 = ScoreWithIndex(4.0, 2);
    assert!(item1 < item2, "Item with smaller score should be smaller");
  }

  #[test]
  fn test_reverse_item_with_higher_score_higher() {
    let item1 = ScoreWithIndex(4.0, 1);
    let item2 = ScoreWithIndex(1.0, 1);
    assert!(item1 > item2, "Item with higher score should be higher");
  }

  #[test]
  fn test_reverse_item_with_same_score_equal() {
    let item1 = ScoreWithIndex(4.0, 1);
    let item2 = ScoreWithIndex(4.0, 2);
    assert!(
      item2 == item1,
      "Item with same score should be equal (index is ignored)"
    );
  }
}
