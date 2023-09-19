use anyhow::Result;

use super::registry::Registry;
use crate::arena::MainModule;

#[derive(Clone)]
pub struct TemplateLoader {
  pub registry: Registry,
  pub module: MainModule,
}

impl TemplateLoader {
  #[tracing::instrument(
    name = "AppkitModuleLoader::load_app_template_code",
    skip(self),
    level = "trace"
  )]
  pub async fn load_app_template_code(&self) -> Result<String> {
    if let MainModule::App { app } = &self.module {
      return self
        .registry
        .fetch_app_template(&app.template.id, &app.template.version)
        .await;
    }
    unreachable!("Invalid template loader config");
  }

  #[tracing::instrument(
    name = "AppkitModuleLoader::load_plugin_template",
    skip(self),
    level = "trace"
  )]
  pub async fn load_plugin_template(&self) -> Result<String> {
    if let MainModule::Plugin { template } = &self.module {
      return self
        .registry
        .fetch_plugin(&template.id, &template.version)
        .await;
    }
    unreachable!("Invalid template loader config");
  }
}
