use diesel::prelude::*;
use serde_json::Value;

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
