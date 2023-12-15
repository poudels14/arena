use core::cmp::Ordering;
use std::collections::BinaryHeap;

use datafusion::arrow::datatypes::ArrowNativeTypeOp;
use derivative::Derivative;
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
  #[inline]
  pub fn untraviolet<'a>(t: SimilarityType) -> &'a dyn SimilarityScorer {
    match t {
      SimilarityType::Dot => &UltravioletDotSimilarity {},
      _ => unimplemented!(),
    }
  }
}

#[derive(Derivative)]
#[derivative(PartialEq)]
pub struct ItemWithScore<T>(
  pub OrderedFloat<f32>,
  #[derivative(PartialEq = "ignore")] pub T,
);

impl<T> Ord for ItemWithScore<T> {
  fn cmp(&self, other: &Self) -> Ordering {
    self.partial_cmp(other).unwrap_or(Ordering::Equal)
  }
}

impl<T> std::cmp::Eq for ItemWithScore<T> {}

impl<T> PartialOrd for ItemWithScore<T> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.0.compare(*other.0).reverse())
  }
}

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
