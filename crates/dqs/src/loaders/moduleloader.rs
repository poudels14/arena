use super::appkit;
use anyhow::{bail, Error};
use deno_core::{
  futures::FutureExt, ModuleLoader, ModuleSourceFuture, ModuleSpecifier,
  ResolutionKind,
};
use deno_core::{ModuleSource, ModuleType, OpState};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use tracing::debug;
use url::Url;

#[derive(Clone)]
pub struct AppkitModuleLoader {
  pub workspace_id: String,
  pub pool: Pool<ConnectionManager<PgConnection>>,
}

impl ModuleLoader for AppkitModuleLoader {
  #[tracing::instrument(skip(self, _kind))]
  fn resolve(
    &self,
    specifier: &str,
    referrer: &str,
    _kind: ResolutionKind,
  ) -> Result<ModuleSpecifier, Error> {
    let specifier = match specifier.starts_with("@appkit/") {
      true => format!("appkit:///{}", specifier),
      false => {
        if specifier.starts_with("@arena/runtime/")
          && is_runtime_module_allowed(specifier, referrer)
        {
          format!("builtin:///{}", specifier)
        } else {
          specifier.to_string()
        }
      }
    };

    match Url::parse(&specifier) {
      Ok(url) => {
        debug!("module resolution not needed");
        Ok(url)
      }
      _ => {
        bail!("Failed to resolve specifier: {:?}", specifier);
      }
    }
  }

  #[tracing::instrument(skip(self))]
  fn load(
    &self,
    module_specifier: &ModuleSpecifier,
    maybe_referrer: Option<ModuleSpecifier>,
    _is_dynamic: bool,
  ) -> Pin<Box<ModuleSourceFuture>> {
    let specifier = module_specifier.clone().to_string();
    let referrer = maybe_referrer
      .clone()
      .map(|r| r.to_string())
      .unwrap_or(".".to_owned());

    let mut this = self.clone();
    async move {
      let module_source = appkit::ModuleSource::parse(&specifier)?;
      let code = match module_source {
        appkit::ModuleSource::WidgetQuery(src) => {
          let query = this.load_widget_query(this.workspace_id.clone(), &src).await?;
          // TODO(sagar): parse query
          query
        },
        _ => {
          format!(r#"
            console.log('this is module:', "{specifier}", ', loaded from:', `{referrer:?}`);
            export default {{}};
          "#)        
        }
      };
      Ok(ModuleSource {
        code: code.as_bytes().into(),
        module_type: ModuleType::JavaScript,
        module_url_found: specifier.clone(),
        module_url_specified: specifier,
      })
    }
    .boxed_local()
  }

  fn prepare_load(
    &self,
    _op_state: Rc<RefCell<OpState>>,
    _module_specifier: &ModuleSpecifier,
    _maybe_referrer: Option<String>,
    _is_dyn_import: bool,
  ) -> Pin<Box<dyn Future<Output = Result<(), Error>>>> {
    async { Ok(()) }.boxed_local()
  }
}

fn is_runtime_module_allowed(specifier: &str, referrer: &str) -> bool {
  // Note(sagar): allow all modules from main file since it's admin code
  if referrer == "file:///@arena/workspace/main" {
    return true;
  }

  specifier == "@arena/runtime/postgres"
}
