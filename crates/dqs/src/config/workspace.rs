use runtime::permissions::NetPermissions;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceConfig {
  #[serde(default)]
  pub runtime: RuntimeConfig,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeConfig {
  // heap limit in MB
  pub heap_limit_mb: Option<usize>,
  pub net_permissions: Option<NetPermissions>,
}
