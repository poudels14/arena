use anyhow::{anyhow, Result};
use common::config::{EnvironmentVariables, JavascriptConfig};
use derivative::Derivative;
use json_patch::merge;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, Default)]
pub struct ServerConfig {
  #[serde(default = "_default_server_entry")]
  #[derivative(Default(value = "_default_server_entry()"))]
  // TODO(sagar): maybe use "package.json".module instead
  pub entry: String,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub javascript: Option<JavascriptConfig>,

  /// env variable override for client
  /// client and server inherit workspace env by default
  #[serde(skip_serializing_if = "Option::is_none")]
  pub env: Option<EnvironmentVariables>,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, Default)]
pub struct ClientConfig {
  #[serde(default = "_default_client_entry")]
  #[derivative(Default(value = "_default_client_entry()"))]
  pub entry: String,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub javascript: Option<JavascriptConfig>,

  /// env variable override for client
  /// client and server inherit workspace env by default
  #[serde(skip_serializing_if = "Option::is_none")]
  pub env: Option<EnvironmentVariables>,
}

/// This is a config for Arena workspace
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, Default)]
pub struct WorkspaceConfig {
  pub name: String,
  pub version: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub env: Option<EnvironmentVariables>,

  #[derivative(Default)]
  #[serde(default = "Default::default")]
  pub server: ServerConfig,

  #[derivative(Default)]
  #[serde(default = "Default::default")]
  pub client: ClientConfig,
}

impl WorkspaceConfig {
  pub fn from_path(filepath: &PathBuf) -> Result<Self> {
    let content =
      fs::read(filepath).map_err(|e| anyhow!("{}: {:?}", e, filepath))?;
    let mut config: Self = toml::from_str(&std::str::from_utf8(&content)?)
      .map_err(|e| anyhow!("{}", e))?;

    // merge client and server env with common env
    if config.env.is_some() {
      let mut client_env = config.env.as_ref().unwrap().0.clone();
      merge(
        &mut client_env,
        &config.client.env.clone().map(|e| e.0).unwrap_or(json!({})),
      );
      config.client.env = Some(EnvironmentVariables(client_env.to_owned()));

      let mut server_env = config.env.as_ref().unwrap().0.clone();
      merge(
        &mut server_env,
        &config.client.env.clone().map(|e| e.0).unwrap_or(json!({})),
      );
      config.client.env = Some(EnvironmentVariables(server_env.to_owned()));
    }

    Ok(config)
  }
}

fn _default_server_entry() -> String {
  String::from("entry-server.tsx")
}

fn _default_client_entry() -> String {
  String::from("entry-client.tsx")
}

mod tests {
  #[test]
  fn test_default_entry_server() {
    let config = crate::WorkspaceConfig {
      ..Default::default()
    };
    assert_eq!(config.server.entry, "entry-server.tsx");
  }

  #[test]
  fn test_serialize_default_entry_server() {
    let config: crate::WorkspaceConfig = toml::from_str(
      r#"
      name = "test-workspace"
    "#,
    )
    .unwrap();

    assert_eq!(config.server.entry, "entry-server.tsx");
  }
}
