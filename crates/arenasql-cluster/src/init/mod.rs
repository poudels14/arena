use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::io::file::File;
use crate::schema::{ClusterBuilder, UserBuilder, MANIFEST_FILE};

#[derive(clap::Parser, Debug, Clone)]
pub struct InitCluster {
  /// Admin user name
  #[arg(long = "user")]
  pub admin_user: String,

  /// Admin user password
  #[arg(long = "password")]
  pub admin_pass: String,

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
        .name(self.admin_user)
        .password(self.admin_pass)
        .build()
        .unwrap(),
    )?;

    let mut manifest_file = File::create(&path.join(MANIFEST_FILE))
      .context("Error creating new manifest file")?;
    manifest_file.write_sync(&cluster)?;
    Ok(())
  }
}
