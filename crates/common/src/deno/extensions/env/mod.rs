use super::BuiltinExtension;
use crate::arena::ArenaConfig;
use crate::deno::RuntimeConfig;
use crate::dotenv;
use anyhow::Result;
use deno_core::op2;
use deno_core::Extension;
use deno_core::ExtensionFileSource;
use deno_core::ExtensionFileSourceCode;
use deno_core::Op;
use deno_core::OpState;
use serde_json::json;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(Extension {
      ops: vec![op_load_env::DECL].into(),
      js_files: vec![ExtensionFileSource {
        specifier: "setup",
        code: ExtensionFileSourceCode::IncludedInBinary(
          r#"
          Object.assign(globalThis.process.env, Arena.core.ops.op_load_env());
          "#,
        ),
      }]
      .into(),
      enabled: true,
      ..Default::default()
    }),
    runtime_modules: vec![],
    snapshot_modules: vec![],
  }
}

#[op2]
#[serde]
fn op_load_env(state: &mut OpState) -> Result<serde_json::Value> {
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
