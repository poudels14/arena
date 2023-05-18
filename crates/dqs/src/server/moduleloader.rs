use super::state::RuntimeState;
use crate::db::widget::{self, widgets};
use crate::loaders;
use crate::specifier::ParsedSpecifier;
use crate::types::widget::{
  DataConfig, SourceConfig, WidgetConfig, WidgetQuerySpecifier,
};
use anyhow::{anyhow, bail, Error, Result};
use deno_core::{
  futures::FutureExt, ModuleLoader, ModuleSourceFuture, ModuleSpecifier,
  ResolutionKind,
};
use deno_core::{ModuleSource, ModuleType, OpState};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use serde_json::{json, Value};
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use tracing::info;
use url::Url;

#[derive(Clone)]
pub struct AppkitModuleLoader {
  pub workspace_id: String,
  pub pool: Pool<ConnectionManager<PgConnection>>,
  pub state: RuntimeState,
}

impl ModuleLoader for AppkitModuleLoader {
  #[tracing::instrument(
    name = "AppkitModuleLoader::resolve",
    skip(self, _kind)
  )]
  fn resolve(
    &self,
    specifier: &str,
    referrer: &str,
    _kind: ResolutionKind,
  ) -> Result<ModuleSpecifier, Error> {
    let is_referrer_main_module =
      referrer == "." || referrer == "file:///@arena/workspace/main";

    // Note(sagar): block all valid Urls as module specifier
    // so that privilege/admin modules can't be loaded from user modules
    if let Ok(url) = Url::parse(&specifier) {
      if is_referrer_main_module {
        return Ok(url);
      }
      info!("Unsupported module specifier: {:?}", specifier);
      bail!("Unsupported module specifier: {:?}", specifier);
    }

    let specifier = match is_referrer_main_module {
      // allow all modules to be loaded by main module since it's admin code
      true => format!("builtin:///{}", specifier),
      // allow all dqs functions modules; those are meant to be used by user
      // code
      false if specifier.starts_with("@arena/functions/") => {
        format!("builtin:///{}", specifier)
      }
      // modules that start with `~` are workspace modules and loaded
      // dynamically, normally from db/cache
      false if specifier.starts_with("~") => {
        format!("workspace:///{}", specifier)
      }
      // relative specifiers are used to load env variables, etc
      // for example |import env from "./env"| to load env
      false if specifier.starts_with("./") => {
        format!("{}/{}", referrer, specifier)
      }
      _ => {
        info!("Unsupported module specifier: {:?}", specifier);
        bail!("Unsupported module specifier: {:?}", specifier)
      }
    };

    Url::parse(&specifier)
      .map_err(|_| anyhow!("Failed to resolve specifier: {:?}", specifier))
  }

  #[tracing::instrument(name = "AppkitModuleLoader::load", skip(self))]
  fn load(
    &self,
    module_specifier: &ModuleSpecifier,
    maybe_referrer: Option<ModuleSpecifier>,
    _is_dynamic: bool,
  ) -> Pin<Box<ModuleSourceFuture>> {
    let specifier = module_specifier.clone().to_string();

    let mut loader = self.clone();
    async move {
      let parsed_specifier = ParsedSpecifier::from(&specifier)?;
      let code = match parsed_specifier {
        ParsedSpecifier::Env { app_id, widget_id } => {
          loader.load_env_variable_module(&app_id, &widget_id).await?
        }
        ParsedSpecifier::WidgetQuery(src) => {
          loader.load_widget_query_module(&src).await?
        }
        _ => bail!("Unsupported module"),
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

impl AppkitModuleLoader {
  async fn load_widget_query_module(
    &mut self,
    specifier: &WidgetQuerySpecifier,
  ) -> Result<String> {
    let connection = &mut self.pool.get()?;
    let widget = widgets::table
      .filter(widgets::id.eq(specifier.widget_id.to_string()))
      .first::<widget::Widget>(connection);

    return match widget {
      Ok(w) => {
        let config: WidgetConfig = serde_json::from_value(w.config)?;
        let data_config = config
          .data
          .get(&specifier.field_name)
          .ok_or(anyhow!("field config not found"))?;

        match &data_config {
          DataConfig::Dynamic { config } | DataConfig::Template { config } => {
            match config {
              SourceConfig::Sql(sql_config) => {
                loaders::sql::from_config(specifier, sql_config)
              },
              SourceConfig::JavaScript(js_config) => {
                loaders::javascript::from_config(specifier, js_config)
              }
            }
          }
        }
      }
      Err(e) => Err(e.into()),
    };
  }

  async fn load_env_variable_module(
    &mut self,
    _app_id: &str,
    _widget_id: &str,
  ) -> Result<String> {
    let variables = self
      .state
      .env_variables
      .iter()
      .map(|(tmp_id, env)| {
        json!({
          "id": tmp_id,
          "key": env.key,
          "type": env.ttype,
          "value": if env.ttype == "secret" { None } else { Some(env.value.clone()) }
        })
      })
      .collect::<Vec<Value>>();
    loaders::env::from_vec(variables)
  }
}
