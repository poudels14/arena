#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricType {
  /// Inner product, also called cosine distance
  InnerProduct = 0,
  /// Euclidean L2-distance
  L2 = 1,
}
