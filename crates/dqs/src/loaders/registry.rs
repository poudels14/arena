use anyhow::{bail, Result};
use derivative::Derivative;
use http::StatusCode;
use std::time::Duration;

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct Registry {
  pub host: String,

  #[derivative(Debug = "ignore")]
  pub api_key: String,
}

impl Registry {
  /// Load the server bundle of the given template_id and version
  #[tracing::instrument(skip(self), level = "debug")]
  pub async fn fetch_app_template(
    &self,
    template_id: &str,
    version: &str,
  ) -> Result<String> {
    self
      .fetch_code(&format!(
        "{}/apps/{}/{}/server/index.js",
        self.host, template_id, version
      ))
      .await
  }

  #[tracing::instrument(skip(self), level = "debug")]
  pub async fn fetch_plugin(
    &self,
    plugin_id: &str,
    version: &str,
  ) -> Result<String> {
    self
      .fetch_code(&format!(
        "{}/server/templates/plugins/{}/{}.js",
        self.host, plugin_id, version,
      ))
      .await
  }

  /// Load a bundle from the registry
  #[tracing::instrument(skip(self), level = "debug")]
  async fn fetch_code(&self, url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let res = client
      .get(url)
      .query(&[("API_KEY", self.api_key.clone())])
      .timeout(Duration::from_secs(30))
      .send()
      .await?;

    if res.status() != StatusCode::OK {
      bail!("Failed to fetch code from registry");
    }

    let bytes = res.bytes().await?;
    Ok(simdutf8::basic::from_utf8(&bytes)?.to_owned())
  }
}
