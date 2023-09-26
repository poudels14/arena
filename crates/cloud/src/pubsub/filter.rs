use crate::identity::Identity;

#[derive(Default, Debug, Clone)]
pub struct EventFilter {
  pub source: Option<Identity>,
}
