use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use arenasql::execution::Privilege;

use crate::schema::{
  ClusterConfigBuilder, UserBuilder, ADMIN_USERNAME, APPS_USERNAME,
};

#[derive(clap::Parser, Debug, Clone)]
pub struct InitCluster {
  /// password for user "arenasql-admin"
  #[arg(default_value = "admin_password")]
  pub admin_password: String,

  /// password for user "arenasql-apps"
  #[arg(default_value = "password")]
  pub apps_password: String,

  /// Path to the directory where database data should be stored
  #[arg(long)]
  pub catalogs_dir: String,

  /// Path to the file where config should be written
  #[arg(long)]
  pub config: String,
}

impl InitCluster {
  pub async fn execute(self) -> Result<()> {
    let config_path = Path::new(&self.config);
    if config_path.exists() {
      bail!("Arenasql config already exists: {:?}", config_path);
    } else {
      if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
      }
    }

    let catalog_dir = Path::new(&self.catalogs_dir);
    if !catalog_dir.exists() {
      fs::create_dir_all(catalog_dir)?;
    }

    let mut config = ClusterConfigBuilder::default()
      .catalogs_dir(catalog_dir.canonicalize()?.to_str().unwrap().to_owned())
      .build()
      .unwrap();
    config.add_user(
      UserBuilder::default()
        .name(ADMIN_USERNAME.to_owned())
        .password(self.admin_password)
        .privilege(Privilege::SUPER_USER)
        .build()
        .unwrap(),
    )?;
    config.add_user(
      UserBuilder::default()
        .name(APPS_USERNAME.to_owned())
        .password(self.apps_password)
        // "arenasql-apps" user shouldn't have any privilege by default
        // The Table privilege will be given to the queries if the
        // Auth header is verified for each query
        .privilege(Privilege::NONE)
        .build()
        .unwrap(),
    )?;

    let toml = toml::to_string(&config).context("error serializing config")?;
    std::fs::write(self.config, toml)
      .context("error writing config to the file")?;
    Ok(())
  }
}
