use crate::arena::ArenaConfig;
use indexmap::map::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
  pub name: String,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub version: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub main: Option<String>,

  #[serde(rename(serialize = "type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub typ: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub module: Option<String>,

  /// Config related to Arena. Only found in Arena app/workspace projects
  pub arena: Option<ArenaConfig>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub dependencies: Option<IndexMap<String, String>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub imports: Option<IndexMap<String, Value>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub exports: Option<Value>,
}
