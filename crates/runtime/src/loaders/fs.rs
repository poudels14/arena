use std::path::PathBuf;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::{anyhow, bail, Error, Result};
use deno_ast::MediaType;
use deno_core::{
  FastString, ModuleLoader, ModuleSource, ModuleSourceFuture, ModuleSpecifier,
  ModuleType, ResolutionKind,
};
use futures::future::FutureExt;

use crate::buildtools::transpiler::SwcTranspiler;
use crate::resolver::ModuleResolver;
use crate::resolver::ResolutionType;
use crate::resolver::Resolver;
use crate::transpiler::ModuleTranspiler;

pub struct FileModuleLoader {
  resolver: Rc<dyn Resolver>,
  transpiler: Option<Rc<dyn ModuleTranspiler>>,
  default_js_transpiler: SwcTranspiler,
}

impl FileModuleLoader {
  pub fn new(
    resolver: Rc<dyn Resolver>,
    transpiler: Option<Rc<dyn ModuleTranspiler>>,
  ) -> Self {
    Self {
      resolver: resolver.clone(),
      transpiler,
      default_js_transpiler: SwcTranspiler::new(resolver),
    }
  }
}

// Note(sagar): copied from deno_core crate
// TODO(sagar): for some reason, this is being called more than once even
// for a single import, fix it?
impl ModuleLoader for FileModuleLoader {
  #[tracing::instrument(skip(self, _kind), level = "debug")]
  fn resolve(
    &self,
    specifier: &str,
    referrer: &str,
    _kind: ResolutionKind,
  ) -> Result<ModuleSpecifier, Error> {
    Ok(ModuleResolver::new(Some(self.resolver.clone())).resolve(
      &specifier,
      referrer,
      // All resolutions using file loader is import type
      ResolutionType::Import,
    )?)
  }

  #[tracing::instrument(
    skip_all,
    fields(
      module_specifier=module_specifier.as_str(),
      referrer=_referrer.map(|r| r.as_str())
    ),
    level = "debug")
  ]
  fn load(
    &self,
    module_specifier: &ModuleSpecifier,
    _referrer: Option<&ModuleSpecifier>,
    _is_dynamic: bool,
  ) -> Pin<Box<ModuleSourceFuture>> {
    let module_specifier = module_specifier.clone();
    let transpiler = self.transpiler.clone();
    let default_js_transpiler = self.default_js_transpiler.clone();
    async move {
      let path = module_specifier.to_file_path().map_err(|_| {
        anyhow!(
          "Provided module specifier \"{}\" is not a file URL.",
          module_specifier
        )
      })?;

      let media_type = MediaType::from_specifier(&module_specifier);
      let (module_type, already_transpiled_code, needs_transpilation) =
        match media_type {
          MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => {
            (ModuleType::JavaScript, None, false)
          }
          MediaType::TypeScript
          | MediaType::Mts
          | MediaType::Cts
          | MediaType::Dts
          | MediaType::Dmts
          | MediaType::Dcts
          | MediaType::Tsx
          | MediaType::Jsx => (ModuleType::JavaScript, None, true),
          MediaType::Json => {
            (ModuleType::JavaScript, Some(self::load_json(&path)?), false)
          }
          _ => match path.extension().and_then(|e| e.to_str()) {
            Some("css") => {
              (ModuleType::JavaScript, Some(self::load_css(&path)?), false)
            }
            _ => bail!("Unknown extension of path: {:?}", path),
          },
        };

      let code = match already_transpiled_code {
        Some(code) => code,
        None => {
          let mut code = std::fs::read_to_string(path.clone())?;
          // To support cjs to esm, require('...') needs to be supported
          // and for require('...'), transpiler module has to be enabled.
          // So, if it's not enabled, don't transpile cjs to esm
          let convert_cjs_to_esm = transpiler.is_some();
          if module_type == ModuleType::JavaScript {
            code = default_js_transpiler.transpile(
              &path,
              &media_type,
              &code,
              convert_cjs_to_esm,
            )?;
          }
          match needs_transpilation {
            true => match transpiler.clone() {
              Some(transpiler) => {
                let fut = transpiler.transpile(&path, &code);
                tokio::pin!(fut);
                fut.await?
              }
              None => bail!(
                "Module {} needs to be transpiled but transpiler not set",
                module_specifier.as_str()
              ),
            },
            _ => code.into(),
          }
        }
      };

      let module = ModuleSource::new(
        module_type,
        FastString::Arc(code.into()),
        &module_specifier,
      );
      Ok(module)
    }
    .boxed_local()
  }
}

fn load_css(path: &PathBuf) -> Result<Arc<str>, Error> {
  let css = std::fs::read_to_string(path.clone())?;
  Ok(format!(r#"export default `{css}`;"#).into())
}

fn load_json(path: &PathBuf) -> Result<Arc<str>, Error> {
  let json = std::fs::read_to_string(path.clone())?;
  Ok(format!(r#"export default JSON.parse(`{json}`);"#).into())
}
