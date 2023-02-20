use crate::config::EnvironmentVariables;
use anyhow::Result;
use deno_core::op;
use deno_core::Extension;
use deno_core::OpState;
use serde_json::Value;

pub fn init(env: Option<EnvironmentVariables>) -> Extension {
  Extension::builder("<arena/env>")
    .ops(vec![op_load_env::decl()])
    .state(move |state| {
      state
        .put::<EnvironmentVariables>(env.clone().unwrap_or(Default::default()));
      Ok(())
    })
    .js(vec![(
      "<arena/env>",
      "Arena.env = Object.assign({}, Deno.core.ops.op_load_env());",
    )])
    .build()
}

#[op]
pub fn op_load_env(state: &mut OpState) -> Result<Value> {
  let vars = state.borrow_mut::<EnvironmentVariables>();
  Ok(vars.0.clone())
}
