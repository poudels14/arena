use super::state::RuntimeState;
use crate::config::{DataConfig, SourceConfig, WidgetConfig};
use crate::db::widget::{self, widgets};
use crate::loaders;
use crate::specifier::{ParsedSpecifier, WidgetQuerySpecifier};
use anyhow::{anyhow, bail, Error, Result};
use deno_core::{
  futures::FutureExt, ModuleLoader, ModuleSourceFuture, ModuleSpecifier,
  ResolutionKind,
};
use deno_core::{ModuleSource, ModuleType};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use loaders::ResourceLoader;
use serde_json::{json, Value};
use std::pin::Pin;
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
    skip(self, _kind),
    level = "trace"
  )]
  fn resolve(
    &self,
    specifier: &str,
    referrer: &str,
    _kind: ResolutionKind,
  ) -> Result<ModuleSpecifier, Error> {
    let is_referrer_main_module =
      referrer == "." || referrer == "file:///@arena/dqs/server";

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
        return Url::parse(referrer)
          .and_then(|r| r.join(&format!("{}/{}", r.path(), specifier)))
          .map_err(|_| {
            anyhow!("Failed to resolve specifier: {:?}", specifier)
          });
      }
      _ => {
        info!("Unsupported module specifier: {:?}", specifier);
        bail!("Unsupported module specifier: {:?}", specifier)
      }
    };

    Url::parse(&specifier)
      .map_err(|_| anyhow!("Failed to resolve specifier: {:?}", specifier))
  }

  #[tracing::instrument(
    name = "AppkitModuleLoader::load",
    skip(self),
    level = "trace"
  )]
  fn load(
    &self,
    module_specifier: &ModuleSpecifier,
    maybe_referrer: Option<&ModuleSpecifier>,
    _is_dynamic: bool,
  ) -> Pin<Box<ModuleSourceFuture>> {
    let mut loader = self.clone();
    let specifier = module_specifier.clone();
    let maybe_referrer = maybe_referrer.cloned();
    async move {
      let parsed_specifier = ParsedSpecifier::from(&specifier.to_string())?;
      let code = match parsed_specifier {
        ParsedSpecifier::Env { app_id, widget_id } => {
          match maybe_referrer {
            Some(referrer) => {
              let referrer = referrer.as_str();
              // make sure the referrer that's requesting the env variables is
              // same app and widget or the main module which has the privilege
              if referrer == "builtin:///@arena/dqs/router" {
              } else {
                let parsed_referrer = ParsedSpecifier::from(&referrer)?;
                match parsed_referrer {
                  ParsedSpecifier::WidgetQuery(src) => {
                    if src.app_id != app_id || src.widget_id != widget_id {
                      bail!("Environment variable access denied")
                    }
                  }
                  _ => unreachable!(),
                }
              }
            }
            _ => bail!("Environment variable access denied"),
          }
          loader.load_env_variable_module(&app_id, &widget_id).await?
        }
        ParsedSpecifier::WidgetQuery(src) => {
          loader.load_widget_query_module(&src).await?
        }
        _ => bail!("Unsupported module"),
      };
      Ok(ModuleSource::new(
        ModuleType::JavaScript,
        code.into(),
        &specifier,
      ))
    }
    .boxed_local()
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
              SourceConfig::Postgres(sql_config) => sql_config.to_dqs_module(),
              SourceConfig::JavaScript(js_config) => js_config.to_dqs_module(),
            }
          }
          _ => unreachable!(
            "Only Postgres/Javascript Dynamic data source supported"
          ),
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
      .0
      .iter()
      .map(|(tmp_id, env)| {
        json!({
          "id": env.id,
          "secretId": tmp_id,
          "key": env.key,
          "isSecret": env.is_secret,
          "value": if env.is_secret { None } else { Some(env.value.clone()) }
        })
      })
      .collect::<Vec<Value>>();
    loaders::env::to_esm_module(variables)
  }
}
