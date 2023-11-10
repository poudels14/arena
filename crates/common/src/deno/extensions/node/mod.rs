mod crypto;
mod digest;

use super::BuiltinExtension;
use crate::resolve_from_root;
use deno_core::{op2, Extension, Op};
use std::path::PathBuf;

pub enum NodeModules {
  Path,
  Process,
  Assert,
  PerfHooks,
  Buffer,
  Crypto,
  Events,
  Fs,
  Tty,
  Util,
}

/// Initialize a node BuiltinExtension
/// If the `None` filter is passed, all node modules are included
/// If `Some(vec![...])` filter is passed, only the modules specified
/// in the filter will be included.
/// Note(sagar): `path`, `process` and `buffer` are always included even
/// when the filter is passed
pub fn extension(module_filter: Option<Vec<&'static str>>) -> BuiltinExtension {
  let required_modules = vec!["path", "process", "buffer"];
  let modules = vec![
    "assert",
    "perf_hooks",
    "crypto",
    "events",
    "fs",
    "fs/promises",
    "tty",
    "util",
    "url",
  ];

  let module_filter = module_filter.unwrap_or(modules.clone());
  BuiltinExtension {
    extension: Some(self::init_ops()),
    runtime_modules: vec![("setup", include_str!("./node.js"))],
    snapshot_modules: [
      required_modules,
      modules
        .iter()
        .filter(|m| module_filter.contains(m))
        .map(|m| m.to_owned())
        .collect(),
    ]
    .concat()
    .iter()
    .map(|m| node_module(m))
    .collect(),
  }
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

fn node_module<'a>(name: &'a str) -> (&'a str, PathBuf) {
  (
    name,
    resolve_from_root!(format!("../../js/arena-runtime/dist/node/{}.js", name)),
  )
}
