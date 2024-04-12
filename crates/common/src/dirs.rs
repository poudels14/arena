use anyhow::{anyhow, Result};
use directories::ProjectDirs;

pub fn portal() -> Result<ProjectDirs> {
  self::from("ai", "useportal", "portal-desktop")
}

pub fn from(
  qualifier: &str,
  organization: &str,
  application: &str,
) -> Result<ProjectDirs> {
  directories::ProjectDirs::from(qualifier, organization, application)
    .ok_or(anyhow!("Failed to get project directory"))
}
