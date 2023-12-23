use anyhow::{anyhow, Result};
use derivative::Derivative;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::env::current_dir;
use std::fs;
use std::path::{Path, PathBuf};

use super::node::{Package, ResolverConfig};
use crate::utils::fs::has_file_in_file_tree;

/// This is a config that arena runtime will use
/// It will be used for workspace config as well as
/// commands like `dagger run`
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, Default)]
pub struct ArenaConfig {
  #[serde(default = "Default::default")]
  pub name: String,

  #[serde(default = "Default::default")]
  pub version: String,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub env: Option<IndexMap<String, String>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub javascript: Option<JavascriptConfig>,

  #[serde(default = "Default::default")]
  pub server: ServerConfig,

  #[serde(default = "Default::default")]
  pub client: ClientConfig,
}

#[allow(unused)]
impl ArenaConfig {
  pub fn load(dir: &Path) -> Result<Self> {
    Self::from_file(&dir.join("package.json"))
  }

  pub fn find_in_path_hierachy() -> Option<ArenaConfig> {
    Self::find_project_root().and_then(|p| Self::load(&p)).ok()
  }

  /// Find a directory with "package.json" in the directory
  /// hierarchy
  /// Defaults to env::current_dir() if config file not found
  pub fn find_project_root() -> Result<PathBuf> {
    let cwd = current_dir()?;
    let maybe_package_dir = has_file_in_file_tree(Some(&cwd), "package.json");

    // If package.json is not found, use current dir
    Ok(maybe_package_dir.clone().unwrap_or(cwd))
  }

  fn from_file(file: &PathBuf) -> Result<Self> {
    let content = fs::read(file).map_err(|e| anyhow!("{}: {:?}", e, file))?;
    let package = serde_json::from_slice::<Package>(&content)?;

    let config = package
      .arena
      .map(|mut a| {
        a.name = package.name;
        a.version = package.version.unwrap_or_default();
        // merge top level env with client/server env
        if let Some(env) = a.env.as_ref() {
          let mut client_env = env.clone();
          client_env.extend(a.client.env.unwrap_or_default());
          a.client.env = Some(client_env);

          let mut server_env = env.clone();
          server_env.extend(a.server.env.unwrap_or_default());
          a.server.env = Some(server_env);
        }

        // merge top level js config with client/server js config
        if let Some(resolve) = a.javascript.clone().and_then(|j| j.resolve) {
          a.client.javascript = Some(JavascriptConfig {
            resolve: Some(
              resolve.clone().merge(
                a.client
                  .javascript
                  .as_ref()
                  .clone()
                  .and_then(|j| j.resolve.clone())
                  .unwrap_or_default(),
              ),
            ),
          });

          a.server.javascript = Some(JavascriptConfig {
            resolve: Some(
              resolve.merge(
                a.server
                  .javascript
                  .as_ref()
                  .clone()
                  .and_then(|j| j.resolve.clone())
                  .unwrap_or_default(),
              ),
            ),
          });
        }

        a
      })
      .unwrap_or_default();
    Ok(config)
  }
}

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
  pub env: Option<IndexMap<String, String>>,
}

fn _default_server_entry() -> String {
  String::from("entry-server.tsx")
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
  pub env: Option<IndexMap<String, String>>,
}

fn _default_client_entry() -> String {
  String::from("entry-client.tsx")
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, Default)]
pub struct JavascriptConfig {
  /// Config related to Javascript and Typescript
  #[serde(skip_serializing_if = "Option::is_none")]
  pub resolve: Option<ResolverConfig>,
}

mod tests {
  #[test]
  fn test_default_entry_server() {
    let config = super::ArenaConfig {
      ..Default::default()
    };
    assert_eq!(config.server.entry, "entry-server.tsx");
  }
}
