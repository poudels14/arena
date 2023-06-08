use super::BuiltinExtension;
use crate::config::ArenaConfig;
use anyhow::Result;
use deno_core::op;
use deno_core::ExtensionFileSource;
use deno_core::ExtensionFileSourceCode;
use deno_core::OpState;
use serde_json::Value;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(self::env::init_ops_and_esm()),
    runtime_modules: vec![],
    snapshot_modules: vec![],
  }
}

deno_core::extension!(
  env,
  ops = [op_load_env],
  customizer = |ext: &mut deno_core::ExtensionBuilder| {
    ext.js(vec![ExtensionFileSource {
      specifier: "setup",
      code: ExtensionFileSourceCode::IncludedInBinary(
        "Arena.env = Object.assign({}, Arena.core.ops.op_load_env());\nObject.assign(globalThis.process.env, Arena.env);",
      ),
    }]);
    ext.force_op_registration();
  }
);

#[op]
fn op_load_env(state: &mut OpState) -> Result<Value> {
  let env = state
    .try_borrow::<ArenaConfig>()
    .and_then(|c| c.env.clone())
    .unwrap_or_default();
  Ok(env.0)
}
