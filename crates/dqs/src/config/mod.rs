use serde::Deserialize;
use std::collections::HashMap;

pub mod workspace;

#[derive(Debug, Deserialize)]
pub struct WidgetConfig {
  pub data: HashMap<String, DataConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "source")]
pub enum DataConfig {
  #[serde(alias = "dynamic")]
  Dynamic { config: SourceConfig },
  #[serde(alias = "template")]
  Template { config: SourceConfig },
  #[serde(alias = "config")]
  Unknown,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "loader")]
pub enum SourceConfig {
  #[serde(alias = "@arena/sql/postgres")]
  Postgres(PostgresSourceConfig),
  #[serde(alias = "@arena/server-function")]
  JavaScript(JavascriptSourceConfig),
}

#[derive(Debug, Deserialize, Default)]
pub struct PostgresSourceConfig {
  /**
   * Resource id of the database
   */
  pub db: String,
  pub value: String,
  #[serde(default = "SqlMetadata::default")]
  pub metadata: SqlMetadata,
}

#[derive(Debug, Deserialize, Default)]
pub struct SqlMetadata {
  pub args: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct JavascriptSourceConfig {
  pub value: String,
  pub metadata: Option<JavascriptQueryMetadata>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavascriptQueryMetadata {
  pub server_module: String,
  pub resources: Option<Vec<String>>,
}
