pub mod env;
pub mod javascript;
pub(crate) mod registry;
pub mod sql;
use anyhow::Result;

pub trait ResourceLoader {
  /// This should return a Javascript ESM module that exports a default
  /// function
  fn to_dqs_module(&self) -> Result<String>;
}
