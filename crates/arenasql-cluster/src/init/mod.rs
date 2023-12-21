use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use arenasql::execution::Privilege;

use crate::io::file::File;
use crate::schema::{
  ClusterBuilder, UserBuilder, ADMIN_USERNAME, APPS_USERNAME, MANIFEST_FILE,
};

#[derive(clap::Parser, Debug, Clone)]
pub struct InitCluster {
  /// password for user "arenasql-admin"
  #[arg(default_value = "admin_password")]
  pub admin_password: String,

  /// password for user "arenasql-apps"
  #[arg(default_value = "password")]
  pub apps_password: String,

  /// Directory to setup workspace in
  #[arg(long)]
  pub dir: String,
}

impl InitCluster {
  pub async fn execute(self) -> Result<()> {
    let path = Path::new(&self.dir);
    if path.join(MANIFEST_FILE).exists() {
      bail!("Arena cluster already exists in: {:?}", path);
    } else {
      fs::create_dir_all(path.join("catalogs"))?;
    }

    let mut cluster = ClusterBuilder::default().build().unwrap();
    cluster.add_user(
      UserBuilder::default()
        .name(ADMIN_USERNAME.to_owned())
        .password(self.admin_password)
        .privilege(Privilege::SUPER_USER)
        .build()
        .unwrap(),
    )?;
    cluster.add_user(
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

    let mut manifest_file = File::create(&path.join(MANIFEST_FILE))
      .context("Error creating new manifest file")?;
    manifest_file.write_sync(&cluster)?;
    Ok(())
  }
}
