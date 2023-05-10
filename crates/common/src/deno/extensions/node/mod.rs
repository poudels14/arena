mod crypto;
mod digest;

use super::extension::BuiltinExtension;
use crate::resolve_from_root;
use deno_core::Extension;
use std::path::PathBuf;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(self::init_ops()),
    runtime_modules: vec![("setup", include_str!("./node.js"))],
    snapshot_modules: vec![
      node_module("path"),
      node_module("process"),
      node_module("assert"),
      node_module("perf_hooks"),
      node_module("buffer"),
      node_module("crypto"),
      node_module("events"),
      node_module("fs"),
      node_module("fs/promises"),
      node_module("tty"),
      node_module("util"),
    ],
  }
}

pub fn init_ops() -> Extension {
  Extension::builder("arena/runtime/node")
    .ops(vec![
      crypto::op_node_create_hash::decl(),
      crypto::op_node_hash_update::decl(),
      crypto::op_node_hash_update_str::decl(),
      crypto::op_node_hash_digest::decl(),
      crypto::op_node_hash_digest_hex::decl(),
      crypto::op_node_generate_secret::decl(),
    ])
    .build()
}

fn node_module<'a>(name: &'a str) -> (&'a str, PathBuf) {
  (
    name,
    resolve_from_root!(format!("../../js/arena-runtime/dist/node/{}.js", name)),
  )
}
