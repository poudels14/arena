use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Node {
  User { id: String },
  App { id: String },
  Workflow { id: String },
  Unknown,
}
