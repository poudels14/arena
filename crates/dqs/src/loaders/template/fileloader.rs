use anyhow::Result;
use async_trait::async_trait;

use super::TemplateLoader;
use crate::arena::MainModule;

#[derive(Clone)]
pub struct FileTemplateLoader {
  pub module: MainModule,
}

#[async_trait]
impl TemplateLoader for FileTemplateLoader {
  async fn load_app_template(&self) -> Result<String> {
    unimplemented!();
  }

  async fn load_plugin_template(&self) -> Result<String> {
    unreachable!("Invalid template loader config");
  }
}
