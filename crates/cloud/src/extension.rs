use crate::jwt::{op_cloud_jwt_sign, op_cloud_jwt_verify};
use crate::transpile::op_cloud_transpile_js_data_query;
use common::deno::extensions::BuiltinExtension;
use deno_core::Extension;

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
