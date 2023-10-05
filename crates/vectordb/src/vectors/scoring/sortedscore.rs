use derivative::Derivative;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

static FLOAT_MARGIN: f32 = 0.00000005;

pub type Score = f32;

#[derive(Derivative)]
#[derivative(PartialEq)]
pub struct ItemWithScore<T>(
  pub Score,
  #[derivative(PartialEq = "ignore")] pub T,
);

/// SortedSimilarityScores sorts and retains top-k scores and its index.
/// Since, higher similarity score mean the vectors are more similar, this
/// uses min-heap to store the top-k (index, score) and will remove lowest
/// scores when a new score is inserted to maintain a list of most similar
/// vector index and their similarity score
pub struct SortedSimilarityScores<T> {
  k: usize,
  heap: BinaryHeap<ItemWithScore<T>>,
}

impl<T> SortedSimilarityScores<T> {
  pub fn new(k: usize) -> Self {
    Self {
      k,
      heap: BinaryHeap::with_capacity(k + 2),
    }
  }

  pub fn push(&mut self, item: (Score, T)) {
    self.heap.push(ItemWithScore(item.0, item.1));
    if self.heap.len() > self.k + 1 {
      self.heap.pop();
    }
  }

  pub fn as_vec(mut self) -> Vec<ItemWithScore<T>> {
    while self.heap.len() > self.k {
      self.heap.pop();
    }
    self.heap.into_sorted_vec()
  }
}

impl<T> Ord for ItemWithScore<T> {
  fn cmp(&self, other: &Self) -> Ordering {
    self.partial_cmp(other).unwrap_or(Ordering::Equal)
  }
}

impl<T> std::cmp::Eq for ItemWithScore<T> {}

impl<T> PartialOrd for ItemWithScore<T> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    let diff = self.0 - other.0;
    if diff > FLOAT_MARGIN {
      Some(Ordering::Less)
    } else if diff < FLOAT_MARGIN {
      Some(Ordering::Greater)
    } else {
      Some(Ordering::Equal)
    }
  }
}

impl<T> FromIterator<ItemWithScore<T>> for Vec<(f32, T)> {
  fn from_iter<U: IntoIterator<Item = ItemWithScore<T>>>(iter: U) -> Self {
    iter.into_iter().map(|item| (item.0, item.1)).collect()
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

    assert_eq!(
      list.as_vec().into_iter().collect::<Vec<(f32, usize)>>(),
      vec![(6., 3), (5., 1)]
    );
  }
}

#[cfg(test)]
mod reverseitem_tests {
  use super::ItemWithScore;

  #[test]
  fn test_score_with_smaller_score_is_bigger() {
    let item1 = ItemWithScore(1.0, 1);
    let item2 = ItemWithScore(4.0, 2);
    assert!(item1 > item2, "Item with smaller score should be bigger");
  }

  #[test]
  fn test_reverse_item_with_higher_score_lower() {
    let item1 = ItemWithScore(4.0, 1);
    let item2 = ItemWithScore(1.0, 1);
    assert!(item1 < item2, "Item with higher score should be smaller");
  }

  #[test]
  fn test_reverse_item_with_same_score_equal() {
    let item1 = ItemWithScore(4.0, 1);
    let item2 = ItemWithScore(4.0, 2);
    assert!(
      item2 == item1,
      "Item with same score should be equal (index is ignored)"
    );
  }
}
