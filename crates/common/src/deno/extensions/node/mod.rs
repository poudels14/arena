mod crypto;
mod digest;

use deno_core::Extension;
use deno_core::ExtensionFileSource;
use deno_core::ExtensionFileSourceCode;

pub fn init_ops() -> Extension {
  Extension::builder("arena/node")
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

pub fn get_modules_for_snapshotting() -> Vec<ExtensionFileSource> {
  vec![
    duplicate_modules(
      vec!["path", "node:path"],
      ExtensionFileSourceCode::LoadedFromFsDuringSnapshot(
        "../../js/arena-runtime/dist/node/path.js".into(),
      ),
    ),
    duplicate_modules(
      vec!["process", "node:process"],
      ExtensionFileSourceCode::LoadedFromFsDuringSnapshot(
        "../../js/arena-runtime/dist/node/process.js".into(),
      ),
    ),
    duplicate_modules(
      vec!["assert", "node:assert"],
      ExtensionFileSourceCode::LoadedFromFsDuringSnapshot(
        "../../js/arena-runtime/dist/node/assert.js".into(),
      ),
    ),
    duplicate_modules(
      vec!["node:perf_hooks"],
      ExtensionFileSourceCode::LoadedFromFsDuringSnapshot(
        "../../js/arena-runtime/dist/node/perf_hooks.js".into(),
      ),
    ),
    duplicate_modules(
      vec!["buffer"],
      ExtensionFileSourceCode::LoadedFromFsDuringSnapshot(
        "../../js/arena-runtime/dist/node/buffer.js".into(),
      ),
    ),
    duplicate_modules(
      vec!["node:crypto"],
      ExtensionFileSourceCode::LoadedFromFsDuringSnapshot(
        "../../js/arena-runtime/dist/node/crypto.js".into(),
      ),
    ),
    duplicate_modules(
      vec!["events", "node:events"],
      ExtensionFileSourceCode::LoadedFromFsDuringSnapshot(
        "../../js/arena-runtime/dist/node/events.js".into(),
      ),
    ),
    duplicate_modules(
      vec!["fs", "node:fs"],
      ExtensionFileSourceCode::LoadedFromFsDuringSnapshot(
        "../../js/arena-runtime/dist/node/fs.js".into(),
      ),
    ),
    duplicate_modules(
      vec!["fs/promises", "node:fs/promises"],
      ExtensionFileSourceCode::LoadedFromFsDuringSnapshot(
        "../../js/arena-runtime/dist/node/fs_promises.js".into(),
      ),
    ),
    duplicate_modules(
      vec!["tty", "node:tty"],
      ExtensionFileSourceCode::LoadedFromFsDuringSnapshot(
        "../../js/arena-runtime/dist/node/tty.js".into(),
      ),
    ),
    duplicate_modules(
      vec!["util", "node:util"],
      ExtensionFileSourceCode::LoadedFromFsDuringSnapshot(
        "../../js/arena-runtime/dist/node/util.js".into(),
      ),
    ),
  ]
  .iter()
  .flatten()
  .map(|s| s.clone())
  .collect()
}

pub fn duplicate_modules(
  specifiers: Vec<&str>,
  code: ExtensionFileSourceCode,
) -> Vec<ExtensionFileSource> {
  specifiers
    .iter()
    .map(|s| ExtensionFileSource {
      specifier: s.to_string(),
      code: code.clone(),
    })
    .collect()
}
