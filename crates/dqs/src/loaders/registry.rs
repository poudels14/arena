use anyhow::{bail, Result};
use http::StatusCode;
use std::time::Duration;

#[derive(Clone)]
pub struct Registry {
  pub host: String,
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
    let client = reqwest::Client::new();
    let res = client
      .get(&format!(
        "{}/server/templates/apps/{}/{}.js",
        self.host, template_id, version
      ))
      .query(&[("API_KEY", self.api_key.clone())])
      .timeout(Duration::from_secs(5))
      .send()
      .await?;

    if res.status() != StatusCode::OK {
      bail!("Failed to fetch app template from registry");
    }

    let bytes = res.bytes().await?;
    Ok(simdutf8::basic::from_utf8(&bytes)?.to_owned())
  }
}
