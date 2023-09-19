use anyhow::Context;
use anyhow::Result;
use common::deno::extensions::BuiltinExtension;
use deno_core::op;
use deno_core::Extension;
use deno_core::ExtensionFileSource;
use deno_core::ExtensionFileSourceCode;
use deno_core::OpState;
use serde_json::json;
use serde_json::Value;
use uuid::Uuid;

use super::ArenaRuntimeState;
use super::MainModule;

#[allow(dead_code)]
pub fn extension() -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(
      Extension::builder("arena/runtime/apps")
        .ops(vec![
          op_arena_get_base_dir::decl(),
          op_arena_load_app_env::decl(),
        ])
        .js(vec![ExtensionFileSource {
          specifier: "setup",
          code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
            "./setup.js"
          )),
        }])
        .force_op_registration()
        .build(),
    ),
    ..Default::default()
  }
}

#[op]
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

#[op]
fn op_arena_load_app_env(state: &mut OpState) -> Result<Value> {
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
