use arenasql::execution::Privilege;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

pub const ADMIN_USERNAME: &'static str = "admin";
pub const APPS_USERNAME: &'static str = "apps";

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct User {
  pub name: String,
  pub password: String,
  pub privilege: Privilege,
}
