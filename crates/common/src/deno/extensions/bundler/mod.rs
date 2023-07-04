use super::BuiltinExtension;
use crate::resolve_from_root;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension {
    extension: None,
    runtime_modules: vec![],
    snapshot_modules: vec![(
      "@arena/runtime/bundler",
      resolve_from_root!("../../js/arena-runtime/dist/bundler.js"),
    )],
  }
}
