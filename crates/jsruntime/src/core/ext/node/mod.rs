mod crypto;
mod digest;

use deno_core::Extension;
use deno_core::ExtensionFileSource;
use deno_core::ExtensionFileSourceCode;

pub fn init() -> Extension {
  Extension::builder("arena/node")
    .ops(vec![
      crypto::op_node_create_hash::decl(),
      crypto::op_node_hash_update::decl(),
      crypto::op_node_hash_update_str::decl(),
      crypto::op_node_hash_digest::decl(),
      crypto::op_node_hash_digest_hex::decl(),
    ])
    .build()
}

pub(crate) fn get_builtin_modules() -> Vec<ExtensionFileSource> {
  vec![
    module(
      vec!["path", "node:path"],
      ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "../../../../../../js/arena-runtime/dist/node/path.js"
      )),
    ),
    module(
      vec!["process", "node:process"],
      ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "../../../../../../js/arena-runtime/dist/node/process.js"
      )),
    ),
    module(
      vec!["assert", "node:assert"],
      ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "../../../../../../js/arena-runtime/dist/node/assert.js"
      )),
    ),
    module(
      vec!["node:perf_hooks"],
      ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "../../../../../../js/arena-runtime/dist/node/perf_hooks.js"
      )),
    ),
    module(
      vec!["node:crypto"],
      ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "../../../../../../js/arena-runtime/dist/node/crypto.js"
      )),
    ),
    module(
      vec!["events", "node:events"],
      ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "../../../../../../js/arena-runtime/dist/node/events.js"
      )),
    ),
    module(
      vec!["fs", "node:fs"],
      ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "../../../../../../js/arena-runtime/dist/node/fs.js"
      )),
    ),
    module(
      vec!["fs/promises", "node:fs/promises"],
      ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "../../../../../../js/arena-runtime/dist/node/fs_promises.js"
      )),
    ),
    module(
      vec!["tty", "node:tty"],
      ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "../../../../../../js/arena-runtime/dist/node/tty.js"
      )),
    ),
    module(
      vec!["util", "node:util"],
      ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "../../../../../../js/arena-runtime/dist/node/util.js"
      )),
    ),
  ]
  .iter()
  .flatten()
  .map(|s| s.clone())
  .collect()
}

fn module(
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
