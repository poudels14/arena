pub mod html;

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
  Extension::builder("arena/cloud/pdf")
    .ops(vec![self::html::op_cloud_pdf_to_html::decl()])
    .force_op_registration()
    .build()
}
