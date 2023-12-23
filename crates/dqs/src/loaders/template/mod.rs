use anyhow::Result;
use async_trait::async_trait;

mod fileloader;
mod registryloader;
pub use fileloader::FileTemplateLoader;
pub use registryloader::RegistryTemplateLoader;

#[async_trait]
pub trait TemplateLoader {
  async fn load_app_template(&self) -> Result<String>;

  async fn load_plugin_template(&self) -> Result<String>;
}
