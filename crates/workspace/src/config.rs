use anyhow::{anyhow, Result};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// This is a config for Arena workspace
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, Default)]
pub struct WorkspaceConfig {
  pub name: String,
  pub version: Option<String>,

  #[serde(default = "_default_server_entry")]
  #[derivative(Default(value = "_default_server_entry()"))]
  // TODO(sagar): maybe use "package.json".module instead
  pub server_entry: String,
}

impl WorkspaceConfig {
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
    let config = crate::WorkspaceConfig {
      ..Default::default()
    };
    assert_eq!(config.server_entry, "entry-server.tsx");
  }

  #[test]
  fn test_serialize_default_entry_server() {
    let config: crate::WorkspaceConfig = toml::from_str(
      r#"
      name = "test-workspace"
    "#,
    )
    .unwrap();

    assert_eq!(config.server_entry, "entry-server.tsx");
  }
}
