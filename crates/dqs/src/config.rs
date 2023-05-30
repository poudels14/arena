use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct WidgetConfig {
  pub data: HashMap<String, DataConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "source")]
pub enum DataConfig {
  #[serde(alias = "dynamic")]
  Dynamic {
    config: SourceConfig,
  },
  Template {
    config: SourceConfig,
  },
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
}
