use anyhow::{bail, Result};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use super::User;

pub static MANIFEST_FILE: &'static str = "MANIFEST.bin";
pub static SYSTEM_CATALOG_NAME: &'static str = "postgres";

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct Cluster {
  #[builder(default = "vec![]")]
  pub users: Vec<User>,
}

impl Cluster {
  #[inline]
  pub fn has_user(&self, name: &str) -> bool {
    self.users.iter().any(|u| u.name == name)
  }

  pub fn add_user(&mut self, user: User) -> Result<()> {
    if self.has_user(&user.name) {
      bail!("User \"{}\" already exists", user.name);
    }

    self.users.push(user);
    Ok(())
  }
}
