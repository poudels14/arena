mod extension;
use anyhow::anyhow;
pub use extension::extension;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct App {
  pub id: String,
  pub root: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
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
