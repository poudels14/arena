use super::App;
use anyhow::anyhow;
use anyhow::Result;
use common::deno::extensions::BuiltinExtension;
use common::deno::resources::env_variable::EnvironmentVariableStore;
use deno_core::op;
use deno_core::Extension;
use deno_core::ExtensionFileSource;
use deno_core::ExtensionFileSourceCode;
use deno_core::OpState;
use serde_json::json;
use serde_json::Value;
use uuid::Uuid;

#[allow(dead_code)]
pub fn extension(app: App) -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(
      Extension::builder("arena/runtime/apps")
        .ops(vec![op_apps_get_app_dir::decl(), op_apps_load_env::decl()])
        .state(|state| {
          state.put::<App>(app);
        })
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
pub fn op_apps_get_app_dir(state: &mut OpState) -> Result<String> {
  let app = state.borrow::<App>();
  app
    .root
    .canonicalize()
    .map(|p| {
      p.to_str()
        .map(|p| p.to_owned())
        .ok_or(anyhow!("Error getting path string"))
    })
    .map_err(|e| anyhow!("Failed to get root dir of the app. {}", e))?
}

#[op]
fn op_apps_load_env(state: &mut OpState) -> Result<Value> {
  let mut variables = state.borrow::<EnvironmentVariableStore>().to_vec();
  let app = state.borrow::<App>();

  // Note(sagar): add default app related env variables
  vec![
    ("MODE", "production"),
    ("SSR", "true"),
    (
      "ARENA_PUBLISHED_ENTRY_CLIENT",
      &format!(
        "{}/static/templates/apps/{}/{}.js",
        app.registry.host, app.template.id, app.template.version
      ),
    ),
  ]
  .iter()
  .for_each(|(key, value)| {
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
