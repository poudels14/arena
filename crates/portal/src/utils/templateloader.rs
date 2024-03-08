use anyhow::Result;
use async_trait::async_trait;
use dqs::loaders::TemplateLoader;

use crate::utils::assets::PortalAppModules;

#[derive(Clone)]
pub struct PortalTemplateLoader {}

#[async_trait]
impl TemplateLoader for PortalTemplateLoader {
  async fn load_app_template(&self) -> Result<String> {
    let assets = PortalAppModules::new();
    let module = assets
      .get_asset(&format!(
        "workspace-desktop/{}/server/index.js",
        env!("PORTAL_DESKTOP_WORKSPACE_VERSION")
      ))?
      .ok_or_else(|| anyhow::anyhow!("Module not found"))
      .map(|bytes| std::str::from_utf8(bytes.as_ref()).unwrap().to_owned())?;
    Ok(module)
  }

  async fn load_plugin_template(&self) -> Result<String> {
    unreachable!("Invalid template loader config");
  }
}
