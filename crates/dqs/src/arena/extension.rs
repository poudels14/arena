use anyhow::Context;
use anyhow::Result;
use common::deno::extensions::BuiltinExtension;
use deno_core::{
  op2, Extension, ExtensionFileSource, ExtensionFileSourceCode, Op, OpState,
};
use serde_json::json;
use serde_json::Value;
use uuid::Uuid;

use super::ArenaRuntimeState;
use super::MainModule;

#[allow(dead_code)]
pub fn extension() -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(Extension {
      name: "arena/dqs/runtime",
      ops: vec![op_arena_get_base_dir::DECL, op_arena_load_env::DECL].into(),
      js_files: vec![ExtensionFileSource {
        specifier: "setup",
        code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
          "./setup.js"
        )),
      }]
      .into(),
      enabled: true,
      ..Default::default()
    }),
    ..Default::default()
  }
}

#[op2]
#[string]
pub fn op_arena_get_base_dir(state: &mut OpState) -> Result<String> {
  let state = state.borrow::<ArenaRuntimeState>();
  state
    .root
    .as_ref()
    .context("Filesystem access not allowed")?
    .canonicalize()
    .context("Failed to get root dir")
    .and_then(|p| {
      p.to_str()
        .map(|p| p.to_owned())
        .context("Error getting path string")
    })
}

#[op2]
#[serde]
fn op_arena_load_env(state: &mut OpState) -> Result<serde_json::Value> {
  let state = state.borrow::<ArenaRuntimeState>();
  let mut variables = state.env_variables.to_vec();

  let default_vars = match &state.module {
    MainModule::App { app } => {
      // Note(sagar): add default app related env variables
      vec![
        ("MODE", "production".to_owned()),
        ("SSR", "true".to_owned()),
        (
          "ARENA_PUBLISHED_ENTRY_CLIENT",
          format!(
            "{}/static/templates/apps/{}/{}.js",
            state.registry.host, app.template.id, app.template.version
          ),
        ),
      ]
    }
    _ => vec![],
  };

  default_vars.iter().for_each(|(key, value)| {
    variables.push(json!({
      "id": Uuid::new_v4().to_string(),
      "secretId": Value::Null,
      "key": key,
      "isSecret": false,
      "value": value,
    }));
  });

  Ok(json!(variables))
}
