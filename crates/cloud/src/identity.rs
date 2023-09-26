use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Identity {
  #[serde(alias = "user")]
  User { id: String },

  #[serde(alias = "app")]
  App { id: String },

  #[serde(alias = "workflow")]
  Workflow { id: String },

  #[default]
  Unknown,
}
