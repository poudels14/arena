use crate::deno::resolver;
use crate::utils::fs::has_file_in_file_tree;
use anyhow::{anyhow, Result};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env::current_dir;
use std::fs;
use std::path::PathBuf;

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, Default)]
pub struct JavascriptConfig {
  /// Config related to Javascript and Typescript
  #[serde(skip_serializing_if = "Option::is_none")]
  pub resolve: Option<resolver::Config>,
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

  /// Find a directory with "arena.config.toml" in the directory
  /// hierarchy
  /// Defaults to env::current_dir() if config file not found
  pub fn find_project_root() -> Result<PathBuf> {
    let cwd = current_dir()?;
    let maybe_arena_config_dir =
      has_file_in_file_tree(Some(&cwd), "arena.config.toml");

    // If arena.config.toml is not found, use current dir
    Ok(maybe_arena_config_dir.clone().unwrap_or(cwd))
  }
}
