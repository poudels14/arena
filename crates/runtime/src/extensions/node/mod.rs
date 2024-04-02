mod crypto;
mod digest;

use deno_core::{op2, v8, Extension, Op, OpState};

use super::r#macro::js_dist;
use super::{BuiltinExtension, SourceCode};
use crate::config::RuntimeConfig;

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
    ("crypto", js_dist!("/node/crypto.js")),
    ("tty", js_dist!("/node/tty.js")),
    ("util", js_dist!("/node/util.js")),
    ("url", js_dist!("/node/url.js")),
    ("stream", js_dist!("/node/stream.js")),
    ("events", js_dist!("/node/events.js")),
    // Above are required modules
    (
      "node/setup",
      SourceCode::Preserved(include_str!("./node.js")),
    ),
  ];

  modules.extend(
    vec![
      ("assert", js_dist!("/node/assert.js")),
      ("perf_hooks", js_dist!("/node/perf_hooks.js")),
      ("fs", js_dist!("/node/fs.js")),
      ("fs/promises", js_dist!("/node/fs/promises.js")),
      ("constants", js_dist!("/node/constants.js")),
      ("os", js_dist!("/node/os.js")),
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
      op_node_build_arch::DECL,
      op_node_build_os::DECL,
      op_node_process_args::DECL,
      op_node_process_exit::DECL,
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
fn op_node_build_arch() -> String {
  env!("TARGET").split('-').nth(0).unwrap().to_string()
}

#[op2]
#[string]
fn op_node_build_os() -> String {
  env!("TARGET").split('-').nth(2).unwrap().to_string()
}

#[op2]
#[serde]
fn op_node_process_args(state: &mut OpState) -> Vec<String> {
  let config = state.borrow_mut::<RuntimeConfig>();
  config.process_args.clone()
}

#[op2(nofast)]
fn op_node_process_exit(isolate: *mut v8::Isolate) {
  unsafe { isolate.as_ref() }.unwrap().terminate_execution();
}
