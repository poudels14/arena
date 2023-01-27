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
  pub allow_synthetic_default_imports: bool,
  pub es_module_interop: bool,
  pub target: ecma::Version,
  pub module: ecma::Version,
  pub module_resolution: String,
  pub jsx_import_source: String,
  pub jsx: String,
  pub strict: bool,
  pub base_url: String,
  pub paths: IndexMap<String, Vec<String>>,
}
