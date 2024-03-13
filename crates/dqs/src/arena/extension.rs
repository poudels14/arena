use anyhow::Result;
use deno_core::{
  op2, Extension, ExtensionFileSource, ExtensionFileSourceCode, Op, OpState,
};
use runtime::extensions::BuiltinExtension;
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
      ops: vec![op_arena_load_env::DECL].into(),
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
#[serde]
fn op_arena_load_env(state: &mut OpState) -> Result<serde_json::Value> {
  let state = state.borrow::<ArenaRuntimeState>();
  let mut variables = state.env_variables.to_vec();

  let default_vars = match &state.module {
    MainModule::App { .. } => {
      // Note(sagar): add default app related env variables
      vec![
        ("MODE", "production".to_owned()),
        ("SSR", "true".to_owned()),
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
