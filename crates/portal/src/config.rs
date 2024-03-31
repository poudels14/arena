use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use common::dirs;
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

  /// This resets all user data except workspace config
  pub fn reset(mut self) -> Result<()> {
    self.workspace_db_password = None;
    self.save()?;
    self.reset_files()?;
    Ok(())
  }

  pub fn data_dir() -> PathBuf {
    dirs::portal()
      .expect("Unable to determine project directory")
      .data_dir()
      .to_owned()
  }

  pub fn get_config_path() -> PathBuf {
    Self::data_dir().join("config.toml")
  }

  pub fn get_catalogs_dir(&self) -> PathBuf {
    // "/catalogs" suffix is added later
    Self::data_dir().join("common")
  }

  #[allow(unused)]
  pub fn encryption_key() -> Vec<u8> {
    env!("PORTAL_DESKTOP_ENC_KEY").as_bytes().to_owned()
  }

  pub fn reset_files(&self) -> Result<()> {
    let dir = self.get_catalogs_dir();
    if dir.exists() {
      std::fs::remove_dir_all(dir)?;
    }
    Ok(())
  }
}
