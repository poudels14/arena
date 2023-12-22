use derivative::Derivative;
use indexmap::map::IndexMap;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
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

  /// A list of external modules which shouldn't be resolved, esp when bundling
  #[serde(skip_serializing_if = "IndexSet::is_empty")]
  #[serde(default)]
  pub external: IndexSet<String>,

  /// A list of modules to dedupe
  /// Deduping a module (external npm module) will always resolve the module
  /// to the same path inside the `${project root}/node_modules` directory.
  /// See rollup node resolve plugin's dedupe config for more info.
  #[serde(skip_serializing_if = "IndexSet::is_empty")]
  #[serde(default)]
  pub dedupe: IndexSet<String>,
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
      external: if !other.external.is_empty() {
        other.external
      } else {
        self.external
      },
      dedupe: if !other.dedupe.is_empty() {
        other.dedupe
      } else {
        self.dedupe
      },
    }
  }
}
