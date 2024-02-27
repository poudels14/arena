use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Identity {
  #[serde(rename_all = "camelCase")]
  User { id: String, email: Option<String> },

  #[serde(rename_all = "camelCase")]
  App {
    id: String,

    // user id of the app owner
    owner_id: Option<String>,

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

  pub fn to_user_json(&self) -> Result<String> {
    let user = match self {
      Identity::User { id, email } => json!({
        "id": id,
        "email": email
      }),
      Identity::App { owner_id, .. } => json!({
        "id": owner_id
      }),
      _ => json!({
        "id": "public",
      }),
    };
    Ok(serde_json::to_string(&user)?)
  }
}
