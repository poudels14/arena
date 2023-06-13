use super::ResourceLoader;
use crate::config::JavascriptSourceConfig;
use anyhow::{anyhow, Result};
use common::query::DataQuery;

impl ResourceLoader for JavascriptSourceConfig {
  fn to_dqs_module(&self) -> Result<String> {
    self
      .metadata
      .as_ref()
      .map(|m| m.server_module.clone())
      .or_else(|| {
        DataQuery::from(&self.value.clone())
          .and_then(|q| q.get_server_module())
          .ok()
      })
      .ok_or(anyhow!("Failed to parse data query"))
  }
}
