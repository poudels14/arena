use std::path::Path;

use anyhow::Result;
use deno_core::op2;
use deno_core::Extension;
use deno_core::ExtensionFileSource;
use deno_core::ExtensionFileSourceCode;
use deno_core::Op;
use deno_core::OpState;
use indexmap::IndexMap;
use serde_json::json;

use crate::config::RuntimeConfig;
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
fn op_load_env(state: &mut OpState) -> Result<serde_json::Value> {
  let mut env_vars = IndexMap::new();

  std::env::vars().for_each(|(key, value)| {
    if !env_vars.contains_key(&key) {
      env_vars.insert(key, value);
    }
  });

  let mode = std::env::var("MODE").ok().or(env_vars.get("MODE").cloned());
  if let Some(config) = state.try_borrow::<RuntimeConfig>() {
    // load env variables from .env files
    load_env_from_env_file(&mode.unwrap_or_default(), &config.project_root)
      .unwrap_or(vec![])
      .iter()
      .for_each(|(key, value)| {
        env_vars.insert(key.to_owned(), value.to_owned());
      });
  }

  Ok(json!(env_vars))
}

pub fn load_env_from_env_file(
  mode: &str,
  root: &Path,
) -> Option<Vec<(String, String)>> {
  if mode == "production" {
    dotenvy::from_filename_iter(root.join(".env")).ok()
  } else {
    dotenvy::from_filename_iter(root.join(".env.dev")).ok()
  }
  .map(|envs| {
    envs
      .into_iter()
      .filter(|e| e.is_ok())
      .map(|e| e.unwrap())
      .collect()
  })
}
