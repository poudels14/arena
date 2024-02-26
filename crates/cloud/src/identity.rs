use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Identity {
  #[serde(rename_all = "camelCase")]
  User { id: String, email: Option<String> },

  #[serde(rename_all = "camelCase")]
  App {
    id: String,

    /// Whether the request was originated from user code or Arena system
    /// if `system_originated` is true, it will have "admin" privileges
    #[serde(default)]
    system_originated: Option<bool>,
    // TODO(sagar): figure out a way to make sure JWT identity can't be reused
  },

  #[serde(rename_all = "camelCase")]
  WorkflowRun {
    id: String,

    /// Whether the request was originated from user code or Arena system
    /// if `system_originated` is true, it will have "admin" privileges
    #[serde(default)]
    system_originated: Option<bool>,
    // TODO(sagar): figure out a way to make sure JWT identity can't be reused
  },

  #[default]
  Unknown,
}

impl Identity {
  pub fn system_originated(&self) -> bool {
    match self {
      Identity::App {
        system_originated, ..
      } => system_originated,
      Identity::WorkflowRun {
        system_originated, ..
      } => system_originated,
      _ => &None,
    }
    .unwrap_or(false)
  }

  pub fn to_json(&self) -> Result<String> {
    match self {
      Identity::Unknown => Ok(serde_json::to_string(&Identity::User {
        id: "public".to_owned(),
        email: None,
      })?),
      _ => Ok(serde_json::to_string(self)?),
    }
  }
}
