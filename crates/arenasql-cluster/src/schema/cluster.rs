use anyhow::{bail, Result};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use super::User;

pub static SYSTEM_CATALOG_NAME: &'static str = "postgres";
pub static SYSTEM_SCHEMA_NAME: &'static str = "arena_schema";

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct ClusterConfig {
  /// Path to the directory where database data is stored
  pub catalogs_dir: String,

  #[builder(default = "vec![]")]
  pub users: Vec<User>,

  /// Cache size per database in MB
  #[builder(default = "10")]
  pub cache_size_mb: usize,

  /// A JWT signing secret that's used to authorize queries
  /// that access non-admin databases.
  /// If it's not set, env variable `ARENA_JWT_SECRET` will be checked
  /// and if that's also not set, unauthorized error will be returned
  /// for those queries.
  #[builder(default)]
  pub jwt_secret: Option<String>,

  /// Directory to backup database to
  /// If set, all the database that were opened by the cluster will be
  /// backed up to that directory periodically
  #[builder(default)]
  pub backup_dir: Option<String>,

  /// Directory to put a checkpoint of the databases to
  /// When cluster is terminated, all the databases that were opened will
  /// be checkpointed to that directory
  #[builder(default)]
  pub checkpoint_dir: Option<String>,
}

impl ClusterConfig {
  #[inline]
  pub fn get_user(&self, name: &str) -> Option<&User> {
    self.users.iter().find(|u| u.name == name)
  }

  #[inline]
  pub fn has_user(&self, name: &str) -> bool {
    self.get_user(name).is_some()
  }

  pub fn add_user(&mut self, user: User) -> Result<()> {
    if self.has_user(&user.name) {
      bail!("User \"{}\" already exists", user.name);
    }

    self.users.push(user);
    Ok(())
  }
}
