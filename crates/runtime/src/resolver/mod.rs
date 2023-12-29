use std::rc::Rc;

use deno_core::{ModuleResolutionError, ModuleSpecifier};
use derive_new::new;
use serde::Deserialize;
use tracing::{debug, error};
use url::Url;

mod fs;
pub use fs::FilePathResolver;

#[derive(Debug, PartialEq, Deserialize)]
pub enum ResolutionType {
  // Use this if the npm module is being resolved by `require(...)`
  // for which CJS module needs to be resolved
  Require,
  Import,
}

pub trait Resolver {
  fn resolve(
    &self,
    specifier: &str,
    base: &str,
    resolution_type: ResolutionType,
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
    resolution_type: ResolutionType,
  ) -> Result<ModuleSpecifier, ModuleResolutionError> {
    // TODO(sagar): cache the resolved module specifier?
    let specifier = specifier.strip_prefix("node:").unwrap_or(specifier);
    match Url::parse(&specifier) {
      // 1. Apply the URL parser to specifier.
      //    If the result is not failure, return he result.
      Ok(url) => {
        debug!("module resolution not needed: {}", specifier);
        Ok(url)
      }
      Err(err) => match self.extension.as_ref() {
        // If it wasn't a builtin module, use resolver extension if its set
        Some(ext) => ext.resolve(&specifier, base, resolution_type),
        _ => {
          error!("Parsing specifier failed! specifier = {specifier:?}");
          Err(ModuleResolutionError::InvalidUrl(err))
        }
      },
    }
  }
}
