use std::pin::Pin;
use std::rc::Rc;

use anyhow::{bail, Result};
use deno_ast::ModuleSpecifier;
use deno_core::{
  ModuleLoader, ModuleResolutionError, ModuleSourceFuture, ResolutionKind,
};
use derive_new::new;
use futures::{Future, FutureExt};
use tracing::{debug, error};
use url::Url;

#[derive(new)]
pub struct DefaultModuleLoader {
  builtin_modules: Vec<String>,
  extension: Option<Rc<dyn ModuleLoader>>,
}

impl ModuleLoader for DefaultModuleLoader {
  fn resolve(
    &self,
    specifier: &str,
    referrer: &str,
    kind: ResolutionKind,
  ) -> Result<ModuleSpecifier> {
    // TODO(sagar): cache the resolved module specifier?

    let specifier = specifier.strip_prefix("node:").unwrap_or(specifier);
    let mut specifier = specifier.to_owned();
    if self.builtin_modules.contains(&specifier)
      || specifier.starts_with("@arena/runtime/")
    {
      debug!("Using builtin module: {specifier}");
      specifier = format!("builtin://{}", specifier);
    }
    match Url::parse(&specifier) {
      // 1. Apply the URL parser to specifier.
      //    If the result is not failure, return he result.
      Ok(url) => {
        debug!("module resolution not needed");
        Ok(url)
      }
      Err(err) => match self.extension.as_ref() {
        // If it wasn't a builtin module, use resolver extension if its set
        Some(ext) => Ok(ext.resolve(&specifier, referrer, kind)?),
        _ => {
          error!("Parsing specifier failed! specifier = {specifier:?}");
          Err(ModuleResolutionError::InvalidUrl(err).into())
        }
      },
    }
  }

  fn load(
    &self,
    module_specifier: &ModuleSpecifier,
    maybe_referrer: Option<&ModuleSpecifier>,
    is_dyn_import: bool,
  ) -> Pin<Box<ModuleSourceFuture>> {
    let specifier_str = module_specifier.as_str().to_owned();
    match self.extension.as_ref() {
      Some(loader) => {
        loader.load(module_specifier, maybe_referrer, is_dyn_import)
      }
      _ => async move {
        bail!(
          "Module loading not enabled. Trying to load: {:?}",
          specifier_str
        )
      }
      .boxed_local(),
    }
  }

  fn prepare_load(
    &self,
    module_specifier: &ModuleSpecifier,
    maybe_referrer: Option<String>,
    is_dyn_import: bool,
  ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
    match self.extension.as_ref() {
      Some(loader) => {
        loader.prepare_load(module_specifier, maybe_referrer, is_dyn_import)
      }
      _ => async { Ok(()) }.boxed_local(),
    }
  }
}
