use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
  pub name: String,
  pub version: Option<String>,
}

impl Config {
  pub fn from_path(filepath: &PathBuf) -> Result<Self> {
    let yaml =
      fs::read(filepath).map_err(|e| anyhow!("{}: {:?}", e, filepath))?;
    serde_yaml::from_str(&std::str::from_utf8(&yaml)?)
      .map_err(|e| anyhow!("{}", e))
  }
}
