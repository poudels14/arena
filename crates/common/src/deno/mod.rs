use std::path::PathBuf;

pub mod extensions;
pub mod fetch;
pub mod loader;
pub mod permissions;
pub mod resolver;
pub mod resources;
pub mod utils;

#[derive(Debug)]
pub struct RuntimeConfig {
  pub project_root: PathBuf,
}
