use super::ecma;
use indexmap::map::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TsConfig {
  pub compiler_options: CompilerOptions,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilerOptions {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub strict: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub es_module_interop: Option<bool>,
  pub target: ecma::Version,
  pub module: ecma::Version,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub module_resolution: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub jsx_import_source: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub jsx: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub base_url: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub paths: Option<IndexMap<String, Vec<String>>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_synthetic_default_imports: Option<bool>,
}
