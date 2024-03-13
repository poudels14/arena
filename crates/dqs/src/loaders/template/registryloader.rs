use anyhow::Result;
use async_trait::async_trait;

use super::TemplateLoader;
use crate::arena::MainModule;
use crate::loaders::Registry;

#[derive(Clone)]
pub struct RegistryTemplateLoader {
  pub registry: Registry,
}

#[async_trait]
impl TemplateLoader for RegistryTemplateLoader {
  #[tracing::instrument(
    name = "AppkitModuleLoader::load_app_template_code",
    skip(self),
    level = "trace"
  )]
  async fn load_app_template(&self, module: &MainModule) -> Result<String> {
    if let MainModule::App { app } = module {
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
  async fn load_plugin_template(&self, module: &MainModule) -> Result<String> {
    if let MainModule::PluginWorkflowRun { workflow } = module {
      return self
        .registry
        .fetch_plugin(&workflow.plugin.id, &workflow.plugin.version)
        .await;
    }
    unreachable!("Invalid template loader config");
  }
}
