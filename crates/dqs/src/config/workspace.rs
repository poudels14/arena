use runtime::permissions::NetPermissions;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceConfig {
  pub runtime: RuntimeConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeConfig {
  pub net_permissions: Option<NetPermissions>,
}
