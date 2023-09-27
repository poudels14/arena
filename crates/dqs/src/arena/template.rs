use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
  pub id: String,
  pub version: String,
}

impl TryFrom<Value> for Template {
  type Error = anyhow::Error;
  fn try_from(value: Value) -> Result<Self, Self::Error> {
    serde_json::from_value(value).map_err(|e| anyhow!("{}", e))
  }
}
