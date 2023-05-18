use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug)]
pub struct WidgetQuerySpecifier {
  pub app_id: String,
  pub widget_id: String,
  pub field_name: String,
}

#[derive(Debug, Deserialize)]
pub struct WidgetConfig {
  pub data: HashMap<String, DataConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
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
#[serde(tag = "queryType")]
pub enum SourceConfig {
  #[serde(alias = "sql")]
  Sql(SqlSourceConfig),
  #[serde(alias = "javascript")]
  JavaScript(JavascriptSourceConfig),
}

#[derive(Debug, Deserialize)]
pub struct SqlSourceConfig {
  pub source: String,
  pub db: String,
  pub args: Vec<String>,
  pub query: String,
}

#[derive(Debug, Deserialize)]
pub struct JavascriptSourceConfig {
  pub source: String,
  pub args: Vec<String>,
  pub query: String,
}