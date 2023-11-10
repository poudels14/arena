use anyhow::Result;
use common::query::DataQuery;
use deno_core::{op2, OpState};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataQueryMetadata {
  pub errors: Option<Value>,
  pub props_generator: String,
  pub server_module: String,
  pub resources: Vec<String>,
}

#[op2(async)]
#[serde]
pub async fn op_cloud_transpile_js_data_query(
  _: Rc<RefCell<OpState>>,
  #[string] code: String,
) -> Result<DataQueryMetadata> {
  let query = DataQuery::from(&code)?;
  Ok(DataQueryMetadata {
    errors: None,
    props_generator: query.get_props_generator()?,
    server_module: query.get_server_module()?,
    resources: vec![],
  })
}
