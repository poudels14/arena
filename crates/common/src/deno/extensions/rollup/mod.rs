use super::extension::BuiltinExtension;
use crate::resolve_from_root;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension {
    extension: None,
    runtime_modules: vec![],
    snapshot_modules: vec![(
      "@arena/runtime/rollup",
      resolve_from_root!("../../js/arena-runtime/dist/rollup.js"),
    )],
  }
}
