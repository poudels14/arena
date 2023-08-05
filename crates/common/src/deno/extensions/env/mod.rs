use super::BuiltinExtension;
use crate::arena::ArenaConfig;
use crate::deno::RuntimeConfig;
use crate::dotenv;
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
        r#"
        Object.assign(globalThis.process.env, Arena.core.ops.op_load_env());
        "#,
      ),
    }]);
    ext.force_op_registration();
  }
);

#[op]
fn op_load_env(state: &mut OpState) -> Result<Value> {
  let mut env_vars = {
    state
      .try_borrow::<ArenaConfig>()
      .as_ref()
      .and_then(|c| c.env.clone())
      .unwrap_or_default()
  };

  std::env::vars().for_each(|(key, value)| {
    if !env_vars.contains_key(&key) {
      env_vars.insert(key, value);
    }
  });

  let mode = std::env::var("MODE").ok().or(env_vars.get("MODE").cloned());
  if let Some(config) = state.try_borrow::<RuntimeConfig>() {
    // load env variables from .env files
    dotenv::load_env(&mode.unwrap_or_default(), &config.project_root)
      .unwrap_or(vec![])
      .iter()
      .for_each(|(key, value)| {
        env_vars.insert(key.to_owned(), value.to_owned());
      });
  }

  Ok(json!(env_vars))
}
