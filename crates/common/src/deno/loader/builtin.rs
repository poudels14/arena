use anyhow::{bail, Result};
use deno_ast::ModuleSpecifier;
use deno_core::{ModuleLoader, ModuleSourceFuture, ResolutionKind};
use futures::FutureExt;
use std::pin::Pin;
use url::Url;

pub struct BuiltInModuleLoader {}

impl ModuleLoader for BuiltInModuleLoader {
  fn resolve(
    &self,
    specifier: &str,
    _referrer: &str,
    _kind: ResolutionKind,
  ) -> Result<ModuleSpecifier> {
    let specifier = specifier.strip_prefix("node:").unwrap_or(specifier);
    // Note(sagar): since all modules during build are builtin modules,
    // add url schema `builtin:///` prefix
    let specifier = match specifier.starts_with("builtin:///") {
      true => specifier.to_string(),
      false => format!("builtin:///{}", specifier),
    };

    match Url::parse(&specifier) {
      Ok(url) => Ok(url),
      _ => {
        bail!("Failed to resolve specifier: {:?}", specifier);
      }
    }
  }

  fn load(
    &self,
    module_specifier: &ModuleSpecifier,
    maybe_referrer: Option<&ModuleSpecifier>,
    _is_dynamic: bool,
  ) -> Pin<Box<ModuleSourceFuture>> {
    let specifier = module_specifier.clone();
    let referrer = maybe_referrer.as_ref().map(|r| r.to_string());
    async move {
      bail!(
        "Module loading not supported: specifier = {:?}, referrer = {:?}",
        specifier.as_str(),
        referrer
      );
    }
    .boxed_local()
  }
}
