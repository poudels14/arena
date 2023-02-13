use super::ModuleLoaderConfig;
use crate::buildtools::transpiler;
use crate::core::resolvers;
use crate::{IsolatedRuntime, RuntimeConfig};
use anyhow::{anyhow, bail, Error};
use deno_ast::MediaType;
use deno_core::{
  ModuleLoader, ModuleSource, ModuleSourceFuture, ModuleSpecifier, ModuleType,
  ResolutionKind,
};
use futures::future::FutureExt;
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;

pub struct FsModuleLoader {
  transpile: bool,
  config: Box<ModuleLoaderConfig>,
  runtime: Option<Rc<RefCell<IsolatedRuntime>>>,
}

pub struct ModuleLoaderOption {
  /// whether to auto-transpile the code when loading
  pub transpile: bool,

  pub config: ModuleLoaderConfig,
}

impl FsModuleLoader {
  pub fn new(option: ModuleLoaderOption) -> Self {
    let runtime = match option.transpile {
      true => Some(Rc::new(RefCell::new(
        IsolatedRuntime::new(RuntimeConfig {
          enable_console: true,
          enable_build_tools: true,
          disable_module_loader: true,
          ..Default::default()
        })
        .unwrap(),
      ))),
      false => None,
    };
    Self {
      transpile: option.transpile,
      config: Box::new(option.config),
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
    let specifier = self.resolve_alias(specifier);
    Ok(resolvers::fs::resolve_import(&specifier, referrer)?)
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
        anyhow!(
          "Provided module specifier \"{}\" is not a file URL.",
          module_specifier
        )
      })?;

      let (module_type, _should_transpile) = match MediaType::from(&path) {
        MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => {
          (ModuleType::JavaScript, false)
        }
        MediaType::Jsx => (ModuleType::JavaScript, transpile),
        MediaType::TypeScript
        | MediaType::Mts
        | MediaType::Cts
        | MediaType::Dts
        | MediaType::Dmts
        | MediaType::Dcts
        | MediaType::Tsx => (ModuleType::JavaScript, transpile),
        MediaType::Json => (ModuleType::Json, false),
        _ => bail!("Unknown extension {:?}", path.extension()),
      };

      let code = std::fs::read_to_string(path.clone())?;
      // Note(sagar): transpile all JS files if transpile is enabled
      // so that even cjs modules are transformed to es6
      let code = match transpile {
        true => {
          let media_type = MediaType::from(&path);
          transpiler::transpile(runtime.unwrap(), &path, &media_type, &code)?
        }
        false => code,
      };

      let module = ModuleSource {
        code: code.as_bytes().into(),
        module_type,
        module_url_specified: module_specifier.to_string(),
        module_url_found: module_specifier.to_string(),
      };
      Ok(module)
    }
    .boxed_local()
  }
}

impl FsModuleLoader {
  fn resolve_alias(&self, specifier: &str) -> String {
    // Note(sagar): if module loader is used, config should be present
    let config = &self.config;
    config
      .alias
      .as_ref()
      .and_then(|alias| {
        for k in alias.keys() {
          if specifier.starts_with(k) {
            let value = alias.get(k).unwrap();
            return Some(format!(
              "{}{}",
              if value.starts_with(".") {
                format!("{}", config.project_root.join(value).to_str().unwrap())
              } else {
                value.to_string()
              },
              &specifier[k.len()..]
            ));
          }
        }
        Some(specifier.to_owned())
      })
      .unwrap_or(specifier.to_owned())
  }
}
