#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum QueryClient {
  Unknown,
  Authenticated { id: String },
  New { user: String, database: String },
}

impl Default for QueryClient {
  fn default() -> Self {
    Self::Unknown
  }
}
