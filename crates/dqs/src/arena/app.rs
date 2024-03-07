use serde::Serialize;

use super::template::Template;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct App {
  pub workspace_id: String,
  pub id: String,
  pub template: Template,
  pub owner_id: Option<String>,
}
