use crate::vector::VectorId;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
  pub distances: Vec<f32>,
  pub labels: Vec<VectorId>,
}
