use anyhow::{anyhow, Result};
use derivative::Derivative;
use indexmap::map::IndexMap;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, Default)]
pub struct ResolverConfig {
  /// to use pnpm, preserve_symlink should be false since packages
  /// are hoisted
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub preserve_symlink: Option<bool>,

  /// Module resolve alias as used by node resolvers, ViteJs, etc
  #[serde(skip_serializing_if = "IndexMap::is_empty")]
  #[serde(default)]
  pub alias: IndexMap<String, String>,

  /// A list of conditions that should be used when resolving modules using
  /// exports field in package.json
  /// similar to exportConditions option for @rollup/plugin-node-resolve
  #[serde(skip_serializing_if = "IndexSet::is_empty")]
  #[serde(default)]
  pub conditions: IndexSet<String>,

  /// A list of modules to dedupe
  /// Deduping a module (external npm module) will always resolve the module
  /// to the same path inside the `${project root}/node_modules` directory.
  /// See rollup node resolve plugin's dedupe config for more info.
  #[serde(skip_serializing_if = "IndexSet::is_empty")]
  #[serde(default)]
  pub dedupe: IndexSet<String>,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, Default)]
pub struct JavascriptConfig {
  /// Config related to Javascript and Typescript
  #[serde(skip_serializing_if = "Option::is_none")]
  pub resolve: Option<ResolverConfig>,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, Default)]
pub struct EnvironmentVariables(pub Value);

/// This is a config that arena runtime will use
/// It will be used for workspace config as well as
/// commands like `dagger run`
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, Default)]
pub struct ArenaConfig {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub javascript: Option<JavascriptConfig>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub env: Option<EnvironmentVariables>,
}

impl ArenaConfig {
  pub fn from_path(filepath: &PathBuf) -> Result<Self> {
    let content =
      fs::read(filepath).map_err(|e| anyhow!("{}: {:?}", e, filepath))?;
    toml::from_str(&std::str::from_utf8(&content)?)
      .map_err(|e| anyhow!("{}", e))
  }
}

impl ResolverConfig {
  pub fn merge(self, other: ResolverConfig) -> Self {
    Self {
      preserve_symlink: other.preserve_symlink.or(self.preserve_symlink),
      alias: if !other.alias.is_empty() {
        other.alias
      } else {
        self.alias
      },
      conditions: if !other.conditions.is_empty() {
        other.conditions
      } else {
        self.conditions
      },
      dedupe: if !other.dedupe.is_empty() {
        other.dedupe
      } else {
        self.dedupe
      },
    }
  }
}
