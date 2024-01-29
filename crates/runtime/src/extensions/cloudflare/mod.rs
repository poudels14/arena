use deno_core::Extension;

use super::{BuiltinExtension, SourceCode};

pub fn extension() -> BuiltinExtension {
  BuiltinExtension::new(
    Some(self::init_ops()),
    vec![(
      "__STATIC_CONTENT_MANIFEST",
      SourceCode::Preserved(include_str!("./manifest.js")),
    )],
  )
}

pub fn init_ops() -> Extension {
  Extension {
    name: "arena/runtime/cloudflare",
    ops: vec![].into(),
    enabled: true,
    ..Default::default()
  }
}
