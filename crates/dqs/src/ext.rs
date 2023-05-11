use crate::runtime::RuntimeConfig;
use anyhow::Result;
use common::deno::extensions::BuiltinExtension;
use common::resolve_from_root;
use deno_core::op;
use deno_core::Extension;
use deno_core::OpState;
use deno_core::ResourceId;
use std::cell::RefCell;
use std::rc::Rc;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(self::init()),
    runtime_modules: vec![],
    snapshot_modules: vec![(
      "@arena/runtime/dqs",
      resolve_from_root!("../../js/arena-runtime/dist/dqs.js"),
    )],
  }
}

pub(crate) fn init() -> Extension {
  Extension::builder("@arena/dqs")
    .ops(vec![op_dqs_start_workspace_server::decl()])
    .build()
}

#[op]
async fn op_dqs_start_workspace_server(
  state: Rc<RefCell<OpState>>,
  workspace_id: String,
) -> Result<ResourceId> {
  let handle = crate::server::new(RuntimeConfig {
    workspace_id,
    address: "0.0.0.0".to_owned(),
    // TODO(sagar): pick random port
    port: 8001,
    ..Default::default()
  })
  .await?;

  let resource_id = state.borrow_mut().resource_table.add(handle);
  Ok(resource_id)
}
