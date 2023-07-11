use super::App;
use anyhow::anyhow;
use anyhow::Result;
use common::deno::extensions::BuiltinExtension;
use deno_core::op;
use deno_core::Extension;
use deno_core::OpState;

pub fn extension(app: App) -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(
      Extension::builder("arena/runtime/apps")
        .ops(vec![op_apps_get_app_dir::decl()])
        .state(|state| {
          state.put::<App>(app);
        })
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
