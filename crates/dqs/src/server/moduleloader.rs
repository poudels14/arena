use super::state::RuntimeState;
use crate::apps::{App, Template};
use crate::config::{DataConfig, SourceConfig, WidgetConfig};
use crate::db::app::{self, apps};
use crate::db::widget::{self, widgets};
use crate::loaders;
use crate::loaders::registry::Registry;
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
  pub app: Option<App>,
  pub registry: Registry,
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
    let is_referrer_admin_module =
      referrer == "." || referrer == "builtin://main";
    let referrer_url = Url::parse(referrer);

    // Note(sagar): block all module specifier that are valid Urls
    // so that privilege/admin modules can't be loaded from user modules
    if let Ok(url) = Url::parse(specifier) {
      if is_referrer_admin_module {
        return Ok(url);
      }
      // Note(sagar): allow builtin modules to be loaded by builtin modules
      // This is necessary to load builtin modules like `path` and `process`
      if let Ok(referrer) = referrer_url {
        if referrer.scheme() == "builtin" {
          return Ok(url);
        }
      }
      info!("Unsupported module specifier: {:?}", specifier);
      bail!("Unsupported module specifier: {:?}", specifier);
    }

    let specifier = if specifier == "@app/template" {
      format!("app:///{}", specifier)
    } else if referrer_url
      .map(|r| r.scheme() == "builtin")
      .unwrap_or(false)
    {
      // Allow all builtin modules if the referrer is builtin module
      format!("builtin:///{}", specifier)
    } else if is_allowed_builtin_module(specifier) {
      format!("builtin:///{}", specifier)
    } else if specifier.starts_with("@") {
      // modules that start with `@` are workspace modules and loaded
      // dynamically, normally from db/cache
      format!("workspace:///{}", specifier)
    } else if specifier.starts_with("./") {
      // relative specifiers are used to load env variables, etc
      // for example |import env from "./env"| to load env
      return Url::parse(referrer)
        .and_then(|r| r.join(&format!("{}/{}", r.path(), specifier)))
        .map_err(|_| anyhow!("Failed to resolve specifier: {:?}", specifier));
    } else {
      info!("Unsupported module specifier: {:?}", specifier);
      bail!("Unsupported module specifier: {:?}", specifier)
    };

    Url::parse(&specifier)
      .map_err(|_| anyhow!("Failed to resolve specifier: {:?}", specifier))
  }

  #[tracing::instrument(
    name = "AppkitModuleLoader::load",
    skip_all,
    fields(
      module_specifier = module_specifier.as_str(),
      maybe_referrer = maybe_referrer.map(|r| r.as_str()),
      is_dynamic = is_dynamic
    ),
    level = "trace"
  )]
  fn load(
    &self,
    module_specifier: &ModuleSpecifier,
    maybe_referrer: Option<&ModuleSpecifier>,
    is_dynamic: bool,
  ) -> Pin<Box<ModuleSourceFuture>> {
    let mut loader = self.clone();
    let specifier = module_specifier.clone();
    let maybe_referrer = maybe_referrer.cloned();

    async move {
      if specifier.scheme() == "app" && specifier.path() == "/@app/template" {
        return Ok(ModuleSource::new(
          ModuleType::JavaScript,
          loader.load_app_template_code().await?.into(),
          &specifier,
        ));
      }

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

  #[tracing::instrument(
    name = "AppkitModuleLoader::load_env_variable_module",
    skip_all,
    level = "trace"
  )]
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

  #[tracing::instrument(
    name = "AppkitModuleLoader::load_app_template_code",
    skip(self),
    level = "trace"
  )]
  async fn load_app_template_code(&self) -> Result<String> {
    if let Some(app) = &self.app {
      let connection = &mut self.pool.clone().get()?;

      let app = app::table
        .filter(apps::id.eq(app.id.to_string()))
        .filter(apps::archived_at.is_null())
        .first::<app::App>(connection);

      if let Some(template) = app.ok().and_then(|a| a.template) {
        let template: Template = template.try_into()?;
        return self
          .registry
          .fetch_app_template(&template.id, &template.version)
          .await;
      }
    }
    bail!("Failed to load app template");
  }
}

// - allow all dqs functions modules; those are meant to be used by user code
// - allow `path` and `process` node modules
fn is_allowed_builtin_module(specifier: &str) -> bool {
  specifier == "path"
    || specifier == "process"
    || specifier == "@arena/runtime/server"
    || specifier.starts_with("@arena/functions/")
}
