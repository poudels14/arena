use anyhow::Result;

pub(crate) mod env;
pub(crate) mod javascript;
pub(crate) mod moduleloader;
pub(crate) mod registry;
pub(crate) mod sql;
pub(crate) mod template;

pub use moduleloader::AppkitModuleLoader;

pub use registry::Registry;
#[allow(unused)]
pub use template::{
  FileTemplateLoader, RegistryTemplateLoader, TemplateLoader,
};

pub trait ResourceLoader {
  /// This should return a Javascript ESM module that exports a default
  /// function
  fn to_dqs_module(&self) -> Result<String>;
}
