use anyhow::Result;
use deno_core::op2;
use deno_core::Extension;
use deno_core::ExtensionFileSource;
use deno_core::ExtensionFileSourceCode;
use deno_core::Op;
use deno_core::OpState;
use indexmap::IndexMap;
use serde_json::json;

use crate::extensions::BuiltinExtension;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension::new(
    Some(Extension {
      ops: vec![op_load_env::DECL].into(),
      js_files: vec![ExtensionFileSource {
      specifier: "env/setup",
      code: ExtensionFileSourceCode::IncludedInBinary(
        "Object.assign(globalThis.process.env, Arena.core.ops.op_load_env());",
      ),
    }]
      .into(),
      enabled: true,
      ..Default::default()
    }),
    vec![],
  )
}

#[op2]
#[serde]
fn op_load_env(_state: &mut OpState) -> Result<serde_json::Value> {
  let mut env_vars = IndexMap::new();
  std::env::vars().for_each(|(key, value)| {
    if !env_vars.contains_key(&key) {
      env_vars.insert(key, value);
    }
  });
  Ok(json!(env_vars))
}
