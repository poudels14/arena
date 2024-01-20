use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Object {
  pub last_modified: String,
  pub e_tag: Option<String>,
  pub storage_class: Option<String>,
  pub key: String,
  // #[serde(rename = "Owner")]
  /// Bucket owner
  // pub owner: Option<Owner>,
  // #[serde(rename = "Size")]
  /// Size in bytes of the object.
  pub size: u64,
}
