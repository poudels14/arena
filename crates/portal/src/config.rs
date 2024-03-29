use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::utils::encryption::decrypt;
use crate::utils::encryption::encrypt;

static NONCE: &'static [u8; 12] = b"8uakmsytpqhf";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceConfig {
  pub workspace_db_password: Option<String>,
  pub user_id: String,
}

impl WorkspaceConfig {
  pub fn load() -> Result<Self> {
    let config = match fs::read(Self::get_config_path()) {
      Ok(enc_config) => {
        let config_bytes = decrypt(&Self::encryption_key(), NONCE, enc_config);
        let config_str = std::str::from_utf8(&config_bytes)?;
        toml::from_str(&config_str)?
      }
      _ => Self {
        user_id: format!("u-desktop-{}", nanoid::nanoid!()),
        ..Default::default()
      },
    };
    Ok(config)
  }

  pub fn save(&self) -> Result<()> {
    let toml = toml::to_string(&self).context("error serializing config")?;
    let encrypted =
      encrypt(&Self::encryption_key(), NONCE.to_owned(), toml.as_bytes());
    std::fs::write(Self::get_config_path(), encrypted)
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
    ProjectDirs::from("ai", "portal", "portal-desktop-4200")
      .expect("Unable to determine project directory")
      .data_dir()
      .to_owned()
  }

  pub fn get_config_path() -> PathBuf {
    Self::data_dir().join("CONFIG")
  }

  pub fn get_catalogs_dir(&self) -> PathBuf {
    // "/catalogs" suffix is added later
    Self::data_dir().join("common")
  }

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
