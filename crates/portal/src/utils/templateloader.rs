use anyhow::{bail, Result};
use async_trait::async_trait;
use dqs::arena::MainModule;
use dqs::loaders::TemplateLoader;

use crate::utils::assets::PortalAppModules;

#[derive(Clone)]
pub struct PortalTemplateLoader {}

#[async_trait]
impl TemplateLoader for PortalTemplateLoader {
  async fn load_app_template(&self, module: &MainModule) -> Result<String> {
    match module {
      MainModule::App { app } => {
        let assets = PortalAppModules::new();
        let module = assets
          .get_asset(&format!(
            "{}/{}/server/index.js",
            app.template.id, app.template.version
          ))?
          .ok_or_else(|| {
            anyhow::anyhow!(
              "App template not found: {}/{}",
              app.template.id,
              app.template.version
            )
          })
          .map(|bytes| {
            std::str::from_utf8(bytes.as_ref()).unwrap().to_owned()
          })?;
        Ok(module)
      }
      _ => {
        bail!("Unsupport module {:?}", module);
      }
    }
  }

  async fn load_plugin_template(&self, _module: &MainModule) -> Result<String> {
    unreachable!("Invalid template loader config");
  }
}
