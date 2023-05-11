use super::BuiltinExtension;
use crate::config::ArenaConfig;
use anyhow::Result;
use deno_core::op;
use deno_core::Extension;
use deno_core::ExtensionFileSource;
use deno_core::ExtensionFileSourceCode;
use deno_core::OpState;
use serde_json::Value;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(self::init()),
    runtime_modules: vec![],
    snapshot_modules: vec![],
  }
}

fn init() -> Extension {
  Extension::builder("arena/runtime/env")
    .ops(vec![op_load_env::decl()])
    .js(vec![ExtensionFileSource {
      specifier: "setup".to_string(),
      code: ExtensionFileSourceCode::IncludedInBinary(
        "Arena.env = Object.assign({}, Arena.core.ops.op_load_env());",
      ),
    }])
    .build()
}

#[op]
pub fn op_load_env(state: &mut OpState) -> Result<Value> {
  let env = state
    .try_borrow::<ArenaConfig>()
    .and_then(|c| c.env.clone())
    .unwrap_or_default();
  Ok(env.0)
}
