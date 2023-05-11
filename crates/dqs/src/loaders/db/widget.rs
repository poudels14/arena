use diesel::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Queryable, Debug)]
pub struct DataSourceConfig {}

#[derive(Queryable, Debug)]
pub struct DataConfig {
  pub t: String, // type = "dynamic" | "template"
  pub config: DataSourceConfig,
}

#[derive(Queryable, Debug)]
pub struct WidgetConfig {
  pub data: HashMap<String, DataSourceConfig>,
}

#[derive(Queryable, Debug)]
pub struct Widget {
  pub id: String,
  pub name: String,
  pub config: Value,
}

diesel::table! {
  widgets (id) {
    id -> Varchar,
    name -> Varchar,
    config -> Jsonb,
  }
}
