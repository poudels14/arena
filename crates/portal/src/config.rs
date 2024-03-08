use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceConfig {
  pub workspace_db_password: Option<String>,
  pub user_id: String,
}

impl WorkspaceConfig {
  pub fn load() -> Result<Self> {
    let config = match fs::read_to_string(Self::get_config_path()) {
      Ok(config_str) => toml::from_str(&config_str)?,
      _ => Self {
        user_id: format!("u-desktop-{}", nanoid::nanoid!()),
        ..Default::default()
      },
    };
    Ok(config)
  }

  pub fn save(&self) -> Result<()> {
    let toml = toml::to_string(&self).context("error serializing config")?;
    std::fs::write(Self::get_config_path(), toml)
      .context("error writing config to the file")?;
    Ok(())
  }

  pub fn data_dir() -> PathBuf {
    ProjectDirs::from("ai", "portal", "portal-desktop-4200")
      .expect("Unable to determine project directory")
      .data_dir()
      .to_owned()
  }

  pub fn get_config_path() -> PathBuf {
    Self::data_dir().join("common").join("config.toml")
  }

  pub fn get_workspace_root_dir(&self) -> PathBuf {
    Self::data_dir().join("runtime").join("workspace")
  }

  pub fn get_catalogs_dir(&self) -> PathBuf {
    // "/catalogs" suffix is added later
    Self::data_dir().join("common")
  }
}
