use super::BuiltinExtension;
use crate::config::ArenaConfig;
use crate::deno::RuntimeConfig;
use anyhow::Result;
use deno_core::op;
use deno_core::ExtensionFileSource;
use deno_core::ExtensionFileSourceCode;
use deno_core::OpState;
use serde_json::json;
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
  let mut env = {
    state
      .try_borrow::<ArenaConfig>()
      .as_ref()
      .and_then(|c| c.env.clone())
      .unwrap_or_default()
  };
  let env = env.0.as_object_mut().unwrap();

  std::env::vars().for_each(|(key, value)| {
    if !env.contains_key(&key) {
      (*env).insert(key, json!(value));
    }
  });

  // load env variables from .env files
  let config = state.borrow::<RuntimeConfig>().project_root.clone();
  let _ = match env.get("MODE").and_then(|m| m.as_str()) {
    Some("production") => dotenvy::from_filename_iter(config.join(".env")).ok(),
    _ => dotenvy::from_filename_iter(config.join(".env.dev")).ok(),
  }
  .map(|envs| {
    envs
      .into_iter()
      .filter(|e| e.is_ok())
      .map(|e| e.unwrap())
      .for_each(|(key, value)| {
        (*env).insert(key, json!(value));
      });
  });

  Ok(json!(env))
}
