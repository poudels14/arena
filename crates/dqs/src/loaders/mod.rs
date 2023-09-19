pub(crate) mod env;
pub(crate) mod javascript;
pub(crate) mod moduleloader;
pub(crate) mod registry;
pub(crate) mod template;

pub mod sql;
use anyhow::Result;

pub trait ResourceLoader {
  /// This should return a Javascript ESM module that exports a default
  /// function
  fn to_dqs_module(&self) -> Result<String>;
}
