use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeImage {
  // 3.10 version is used
  Python { packages: Option<Vec<String>> },
  Python310 { packages: Option<Vec<String>> },
}
