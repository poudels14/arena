use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeImage {
  // 3.10 version is used
  Python { packages: Option<Vec<String>> },
  Python310 { packages: Option<Vec<String>> },
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileSystem {
  pub connection_string: String,
  pub table_name: String,
  // id of the root directory
  pub root: Option<String>,
  pub enable_write: Option<bool>,
}
