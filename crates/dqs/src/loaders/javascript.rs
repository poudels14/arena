use super::ResourceLoader;
use crate::config::JavascriptSourceConfig;
use anyhow::Result;

impl ResourceLoader for JavascriptSourceConfig {
  fn to_dqs_module(&self) -> Result<String> {
    // TODO(sagar): validate JS before running
    Ok(self.value.clone())
  }
}
