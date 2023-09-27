use serde::{Deserialize, Serialize};

use super::Template;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged, rename_all = "camelCase")]
pub enum WorkflowTemplate {
  Plugin { plugin: Template, slug: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PluginWorkflow {
  /// Workflow run id
  pub id: String,
  /// Plugin template info
  pub plugin: Template,
  /// Workflow slug
  pub slug: String,
}
