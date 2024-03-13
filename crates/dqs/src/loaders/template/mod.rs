use anyhow::Result;
use async_trait::async_trait;

use crate::arena::MainModule;

mod fileloader;
mod registryloader;
pub use fileloader::FileTemplateLoader;
pub use registryloader::RegistryTemplateLoader;

#[async_trait]
pub trait TemplateLoader: Send + Sync {
  async fn load_app_template(&self, module: &MainModule) -> Result<String>;

  async fn load_plugin_template(&self, module: &MainModule) -> Result<String>;
}
