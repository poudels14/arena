use crate::buildtools::transpiler;
use crate::core::resolvers::fs::FsModuleResolver;
use crate::{IsolatedRuntime, RuntimeConfig};
use anyhow::{anyhow, bail, Error};
use deno_ast::MediaType;
use deno_core::{
  ModuleLoader, ModuleSource, ModuleSourceFuture, ModuleSpecifier, ModuleType,
  ResolutionKind,
};
use futures::future::FutureExt;
use std::cell::RefCell;
use std::path::PathBuf;
use std::pin::Pin;
use std::rc::Rc;

pub(crate) struct FsModuleLoader {
  transpile: bool,
  runtime: Option<Rc<RefCell<IsolatedRuntime>>>,

  resolver: FsModuleResolver,
}

pub struct ModuleLoaderOption {
  /// whether to auto-transpile the code when loading
  pub transpile: bool,

  pub resolver: FsModuleResolver,
}

impl FsModuleLoader {
  pub fn new(option: ModuleLoaderOption) -> Self {
    let runtime = match option.transpile {
      true => Some(Rc::new(RefCell::new(
        IsolatedRuntime::new(RuntimeConfig {
          enable_console: true,
          // Note(sagar): build tools is needed to transpile when loading
          // modules
          enable_build_tools: true,
          enable_node_modules: true,
          // Note(sagar): since rollup is loaded as side-module when build
          // tools is enabled and rollup needs node modules,
          // need to enable module loader
          disable_module_loader: false,
          transpile: false,
          ..Default::default()
        })
        .unwrap(),
      ))),
      false => None,
    };
    Self {
      transpile: option.transpile,
      resolver: option.resolver,
      runtime,
    }
  }
}

// Note(sagar): copied from deno_core crate
// TODO(sagar): for some reason, this is being called more than once even
// for a single import, fix it?
impl ModuleLoader for FsModuleLoader {
  #[tracing::instrument(skip(self, _kind))]
  fn resolve(
    &self,
    specifier: &str,
    referrer: &str,
    _kind: ResolutionKind,
  ) -> Result<ModuleSpecifier, Error> {
    Ok(self.resolver.resolve_import(&specifier, referrer)?)
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

      let media_type = MediaType::from_specifier(&module_specifier);
      let (module_type, maybe_code, _should_transpile) = match media_type {
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
        | MediaType::Jsx => (ModuleType::JavaScript, None, transpile),
        MediaType::Json => (ModuleType::Json, None, false),
        _ => match path.extension().and_then(|e| e.to_str()) {
          Some("css") => {
            (ModuleType::JavaScript, Some(self::load_css(&path)?), false)
          }
          _ => bail!("Unknown extension {:?}", path.extension()),
        },
      };

      let code = match maybe_code {
        Some(code) => code,
        None => {
          let code = std::fs::read_to_string(path.clone())?;
          // Note(sagar): transpile all JS files if transpile is enabled
          // so that even cjs modules are transformed to es6
          match transpile {
            true => transpiler::transpile(
              runtime.unwrap(),
              &path,
              &media_type,
              &code,
            )?,
            false => code,
          }
          .as_bytes()
          .into()
        }
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

fn load_css(path: &PathBuf) -> Result<Box<[u8]>, Error> {
  let css = std::fs::read_to_string(path.clone())?;
  Ok(
    format!(
      r#"import styleInject from "style-inject";
      const css = `{css}`;
      styleInject(css);
      export default css;
    "#
    )
    .as_bytes()
    .into(),
  )
}
