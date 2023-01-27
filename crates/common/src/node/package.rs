use indexmap::map::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
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

  #[serde(skip_serializing_if = "Option::is_none")]
  pub dependencies: Option<IndexMap<String, String>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub exports: Option<IndexMap<String, Value>>,
}

// #[derive(Default, Debug, Serialize, Deserialize)]
// pub enum Type {
//   #[serde(rename = "module")]
//   Module,
//   #[default]
//   #[serde(rename = "commonjs")]
//   CommonJs,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub enum Export {
//   String(String),
//   Vec(Vec<String>),
//   Map(Exports),
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Exports {
//   pub node: Option<NodeImport>,
//   pub require: Option<String>,
//   pub import: Option<Import>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub enum NodeImport {
//   String(String),
//   Map(IndexMap<String, String>)
//   // pub types: Option<String>,
//   // pub default: Option<String>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Import {
//   pub types: Option<String>,
//   pub default: Option<String>,
// }
