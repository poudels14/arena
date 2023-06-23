use anyhow::Result;
use common::query::DataQuery;
use deno_core::{op, OpState};
use std::cell::RefCell;
use std::rc::Rc;

#[op]
async fn op_cloud_transpile_js_data_query(
  _state: Rc<RefCell<OpState>>,
  code: String,
) -> Result<Vec<String>> {
  let query = DataQuery::from(&code)?;
  Ok(vec![
    query.get_props_generator()?,
    query.get_server_module()?,
  ])
}
