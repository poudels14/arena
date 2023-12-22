mod crypto;
mod digest;

use deno_core::{op2, Extension, Op};

use super::r#macro::js_dist;
use super::{BuiltinExtension, SourceCode};

/// Initialize a node BuiltinExtension
/// If the `None` filter is passed, all node modules are included
/// If `Some(vec![...])` filter is passed, only the modules specified
/// in the filter will be included.
/// Note(sagar): `path`, `process` and `buffer` are always included even
/// when the filter is passed
pub fn extension(module_filter: Option<Vec<&'static str>>) -> BuiltinExtension {
  let mut modules = vec![
    ("path", js_dist!("/node/path.js")),
    ("process", js_dist!("/node/process.js")),
    ("buffer", js_dist!("/node/buffer.js")),
    // Above are required modules
    ("node/setup", SourceCode::Runtime(include_str!("./node.js"))),
  ];

  modules.extend(
    vec![
      ("assert", js_dist!("/node/assert.js")),
      ("perf_hooks", js_dist!("/node/perf_hooks.js")),
      ("crypto", js_dist!("/node/crypto.js")),
      ("events", js_dist!("/node/events.js")),
      ("fs", js_dist!("/node/fs.js")),
      ("fs/promises", js_dist!("/node/fs/promises.js")),
      ("tty", js_dist!("/node/tty.js")),
      ("util", js_dist!("/node/util.js")),
      ("url", js_dist!("/node/url.js")),
    ]
    .into_iter()
    .filter(|(specifier, _)| {
      module_filter
        .as_ref()
        .map(|filter| filter.contains(specifier))
        .unwrap_or(true)
    }),
  );

  BuiltinExtension::new(Some(self::init_ops()), modules)
}

pub fn init_ops() -> Extension {
  Extension {
    name: "arena/runtime/node",
    ops: vec![
      op_node_build_os::DECL,
      crypto::op_node_create_hash::DECL,
      crypto::op_node_hash_update::DECL,
      crypto::op_node_hash_update_str::DECL,
      crypto::op_node_hash_digest::DECL,
      crypto::op_node_hash_digest_hex::DECL,
      crypto::op_node_generate_secret::DECL,
    ]
    .into(),
    enabled: true,
    ..Default::default()
  }
}

#[op2]
#[string]
fn op_node_build_os() -> String {
  env!("TARGET").split('-').nth(2).unwrap().to_string()
}
