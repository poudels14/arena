use anyhow::{anyhow, Result};
use derivative::Derivative;
use indexmap::map::IndexMap;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, Default)]
pub struct JsBuildConfig {
  /// Module resolve alias as used by node resolvers, ViteJs, etc
  #[serde(skip_serializing_if = "IndexMap::is_empty")]
  #[serde(default)]
  pub alias: IndexMap<String, String>,

  /// A list of conditions that should be used when resolving modules using
  /// exports field in package.json
  /// similar to exportConditions option for @rollup/plugin-node-resolve
  #[serde(skip_serializing_if = "IndexSet::is_empty")]
  #[serde(default)]
  pub export_conditions: IndexSet<String>,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, Default)]
pub struct JavascriptConfig {
  /// Config related to Javascript and Typescript
  #[serde(skip_serializing_if = "Option::is_none")]
  pub build: Option<JsBuildConfig>,
}

/// This is a config that arena runtime will use
/// It will be used for workspace config as well as
/// commands like `dagger run`
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, Default)]
pub struct ArenaConfig {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub javascript: Option<JavascriptConfig>,
}

impl ArenaConfig {
  pub fn from_path(filepath: &PathBuf) -> Result<Self> {
    let content =
      fs::read(filepath).map_err(|e| anyhow!("{}: {:?}", e, filepath))?;
    toml::from_str(&std::str::from_utf8(&content)?)
      .map_err(|e| anyhow!("{}", e))
  }
}
