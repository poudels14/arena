use anyhow::{anyhow, Result};
use derivative::Derivative;
use indexmap::map::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, Default)]
pub struct JsBuildConfig {
  /// Module resolve alias as used by node resolvers, ViteJs, etc
  #[serde(skip_serializing_if = "Option::is_none")]
  pub alias: Option<IndexMap<String, String>>,

  /// Mapping from npm module to the package.json's export that should
  /// be use for this module when resolving
  #[serde(skip_serializing_if = "Option::is_none")]
  pub resolve: Option<IndexMap<String, String>>,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, Default)]
pub struct JavascriptConfig {
  /// Config related to Javascript and Typescript
  #[serde(skip_serializing_if = "Option::is_none")]
  pub build_config: Option<JsBuildConfig>,
}

/// This is Arena config that each workspace will have
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, Default)]
pub struct ArenaConfig {
  pub name: String,
  pub version: Option<String>,

  #[serde(default = "_default_server_entry")]
  #[derivative(Default(value = "_default_server_entry()"))]
  // TODO(sagar): maybe use "package.json".module instead
  pub server_entry: String,

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

fn _default_server_entry() -> String {
  String::from("entry-server.tsx")
}

mod tests {
  #[test]
  fn test_default_entry_server() {
    let config = crate::ArenaConfig {
      ..Default::default()
    };
    assert_eq!(config.server_entry, "entry-server.tsx");
  }

  #[test]
  fn test_serialize_default_entry_server() {
    let config: crate::ArenaConfig = toml::from_str(
      r#"
      name = "test-workspace"
    "#,
    )
    .unwrap();

    assert_eq!(config.server_entry, "entry-server.tsx");
  }
}
