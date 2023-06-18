use anyhow::Result;
use common::deno::extensions::BuiltinExtension;
use common::query::DataQuery;
use deno_core::{op, Extension, OpState};
use std::cell::RefCell;
use std::rc::Rc;

use crate::jwt::{op_cloud_jwt_sign, op_cloud_jwt_verify};

pub fn extension() -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(self::init()),
    runtime_modules: vec![],
    snapshot_modules: vec![],
  }
}

pub(crate) fn init() -> Extension {
  Extension::builder("arena/cloud")
    .ops(vec![
      op_cloud_transpile_js_data_query::decl(),
      op_cloud_jwt_sign::decl(),
      op_cloud_jwt_verify::decl(),
    ])
    .force_op_registration()
    .build()
}

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
