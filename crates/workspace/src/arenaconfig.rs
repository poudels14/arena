use anyhow::{anyhow, Result};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
}

impl ArenaConfig {
  pub fn from_path(filepath: &PathBuf) -> Result<Self> {
    let yaml =
      fs::read(filepath).map_err(|e| anyhow!("{}: {:?}", e, filepath))?;
    serde_yaml::from_str(&std::str::from_utf8(&yaml)?)
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
    let config: crate::ArenaConfig = serde_yaml::from_str(
      r#"
      "name": "test-workspace"
    "#,
    )
    .unwrap();

    assert_eq!(config.server_entry, "entry-server.tsx");
  }
}
