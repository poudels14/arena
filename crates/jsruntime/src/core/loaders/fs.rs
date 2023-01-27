use crate::buildtools::transpiler;
use crate::{IsolatedRuntime, RuntimeConfig};
use anyhow::Error;
use deno_core::error::generic_error;
use deno_core::{
  ModuleLoader, ModuleSource, ModuleSourceFuture, ModuleSpecifier, ModuleType,
  ResolutionKind,
};
use futures::future::FutureExt;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

pub struct FsModuleLoader {
  transpile: bool,
  runtime: Option<Arc<Mutex<IsolatedRuntime>>>,
}

pub struct ModuleLoaderOption {
  /// whether to auto-transpile the code when loading
  pub transpile: bool,
}

impl FsModuleLoader {
  pub fn new(option: &ModuleLoaderOption) -> Self {
    let runtime = match option.transpile {
      true => Some(Arc::new(Mutex::new(IsolatedRuntime::new(RuntimeConfig {
        enable_console: true,
        enable_build_tools: true,
        disable_module_loader: true,
        ..Default::default()
      })))),
      false => None,
    };
    Self {
      transpile: option.transpile,
      runtime,
    }
  }
}

// Note(sagar): copied from deno_core crate
impl ModuleLoader for FsModuleLoader {
  fn resolve(
    &self,
    specifier: &str,
    referrer: &str,
    _kind: ResolutionKind,
  ) -> Result<ModuleSpecifier, Error> {
    Ok(crate::core::resolvers::fs::resolve_import(
      specifier, referrer,
    )?)
  }

  fn load(
    &self,
    module_specifier: &ModuleSpecifier,
    _maybe_referrer: Option<ModuleSpecifier>,
    _is_dynamic: bool,
  ) -> Pin<Box<ModuleSourceFuture>> {
    let module_specifier = module_specifier.clone();

    let transpile = self.transpile;
    let runtime = self.runtime.clone();
    async move {
      let path = module_specifier.to_file_path().map_err(|_| {
        generic_error(format!(
          "Provided module specifier \"{}\" is not a file URL.",
          module_specifier
        ))
      })?;
      let module_type = if let Some(extension) = path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        if ext == "json" {
          ModuleType::Json
        } else {
          ModuleType::JavaScript
        }
      } else {
        ModuleType::JavaScript
      };

      let code = std::fs::read(path.clone())?;
      let code = match transpile {
        true => transpiler::transpile(runtime.unwrap(), &path, &code)?,
        false => code.into_boxed_slice(),
      };

      let module = ModuleSource {
        code,
        module_type,
        module_url_specified: module_specifier.to_string(),
        module_url_found: module_specifier.to_string(),
      };
      Ok(module)
    }
    .boxed_local()
  }
}
