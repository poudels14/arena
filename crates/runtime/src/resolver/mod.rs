use std::rc::Rc;

use deno_core::{ModuleResolutionError, ModuleSpecifier};
use derive_new::new;
use tracing::{debug, error};
use url::Url;

mod fs;
pub use fs::FilePathResolver;

pub trait Resolver {
  fn resolve(
    &self,
    specifier: &str,
    base: &str,
  ) -> Result<ModuleSpecifier, ModuleResolutionError>;
}

#[derive(new)]
pub struct ModuleResolver {
  extension: Option<Rc<dyn Resolver>>,
}

impl Resolver for ModuleResolver {
  fn resolve(
    &self,
    specifier: &str,
    base: &str,
  ) -> Result<ModuleSpecifier, ModuleResolutionError> {
    // TODO(sagar): cache the resolved module specifier?
    let specifier = specifier.strip_prefix("node:").unwrap_or(specifier);
    match Url::parse(&specifier) {
      // 1. Apply the URL parser to specifier.
      //    If the result is not failure, return he result.
      Ok(url) => {
        debug!("module resolution not needed");
        Ok(url)
      }
      Err(err) => match self.extension.as_ref() {
        // If it wasn't a builtin module, use resolver extension if its set
        Some(ext) => ext.resolve(&specifier, base),
        _ => {
          error!("Parsing specifier failed! specifier = {specifier:?}");
          Err(ModuleResolutionError::InvalidUrl(err))
        }
      },
    }
  }
}
