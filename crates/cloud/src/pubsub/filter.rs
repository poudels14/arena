use super::Node;

#[derive(Default, Debug, Clone)]
pub struct EventFilter {
  pub source: Option<Node>,
}
