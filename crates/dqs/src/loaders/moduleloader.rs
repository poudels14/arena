use std::pin::Pin;
use std::sync::Arc;

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
use tracing::info;
use url::Url;

use super::template::TemplateLoader;
use crate::config::{DataConfig, SourceConfig, WidgetConfig};
use crate::db::widget::{self, widgets};
use crate::loaders;
use crate::specifier::{ParsedSpecifier, WidgetQuerySpecifier};

#[derive(Clone)]
pub struct AppkitModuleLoader {
  pub workspace_id: String,
  pub pool: Option<Pool<ConnectionManager<PgConnection>>>,
  pub template_loader: Arc<dyn TemplateLoader>,
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
      referrer == "." || referrer == "builtin:///main";
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
        if referrer.scheme() == "builtin" || referrer.scheme() == "dqs" {
          return Ok(url);
        }
      }
      info!("Unsupported module specifier: {:?}", specifier);
      bail!("Unsupported module specifier: {:?}", specifier);
    }

    let specifier = if specifier.starts_with("@dqs/") {
      // modules that start with `@dqs` are workspace modules and loaded
      // dynamically, normally from db/cache
      format!("dqs:///{}", specifier)
    } else if referrer_url
      .map(|r| r.scheme() == "builtin")
      .unwrap_or(false)
    {
      // Allow all builtin modules if the referrer is builtin module
      format!("builtin://{}", specifier)
    } else if is_allowed_builtin_module(specifier) {
      format!("builtin://{}", specifier)
    } else if specifier.starts_with("./") {
      // relative specifiers are used to load env variables, etc
      // for example |import env from "./env"| to load env
      return Url::parse(referrer)
        .and_then(|r| r.join(&format!("{}/{}", r.path(), specifier)))
        .map_err(|_| anyhow!("Failed to resolve specifier: {:?}", specifier));
    } else {
      info!("Unsupported module specifier: {:?}", specifier);
      bail!(
        "Unsupported module specifier: {:?}, referrer: {}",
        specifier,
        referrer
      )
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
      if specifier.scheme() == "dqs" {
        match specifier.path() {
          "/@dqs/template/app" => {
            return Ok::<ModuleSource, anyhow::Error>(ModuleSource::new(
              ModuleType::JavaScript,
              loader.template_loader.load_app_template().await?.into(),
              &specifier,
            ))
          }
          "/@dqs/template/plugin" => {
            return Ok::<ModuleSource, anyhow::Error>(ModuleSource::new(
              ModuleType::JavaScript,
              loader.template_loader.load_plugin_template().await?.into(),
              &specifier,
            ))
          }
          _ => {}
        }
      }

      let parsed_specifier = ParsedSpecifier::from(&specifier.to_string())?;
      let code = match parsed_specifier {
        // TODO(sagar): remove this since all envs are populated in process.env
        ParsedSpecifier::Env { app_id, widget_id } => {
          match maybe_referrer {
            Some(referrer) => {
              let referrer = referrer.as_str();
              // make sure the referrer that's requesting the env variables is
              // same app and widget or the main module which has the privilege
              if referrer == "builtin:///@arena/dqs/widget-server" {
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
        _ => bail!("Unsupported module: {}", specifier),
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
    // TODO(sagar): instead of loading widget query from db directly,
    // use registry
    let connection = &mut self
      .pool
      .clone()
      .ok_or(anyhow!("Database not initialized"))?
      .get()?;
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
    Ok("export default process.env;".to_string())
  }
}

fn is_allowed_builtin_module(specifier: &str) -> bool {
  // - allow `path`, `process`, `crypto` node modules
  specifier == "path"
    || specifier == "process"
    || specifier == "crypto"
    // allow runtime/server since app templates need it
    || specifier == "@arena/runtime/server"
    || specifier == "@arena/runtime/postgres"
    || specifier == "@arena/dqs/postgres"
    || specifier.starts_with("@arena/cloud/")
}
