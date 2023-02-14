use crate::permissions::PermissionsContainer;
use crate::utils::fs::resolve_from_cwd;
use anyhow::Result;
use deno_core::op;
use deno_core::Extension;
use deno_core::OpState;
use deno_core::ZeroCopyBuf;
use std::path::Path;

pub fn init() -> Extension {
  Extension::builder("<arena/core/fs>")
    .ops(vec![op_read_file_sync::decl()])
    .build()
}

#[op]
pub fn op_read_file_sync(
  state: &mut OpState,
  specifier: String,
) -> Result<ZeroCopyBuf> {
  let resolved_path = resolve_from_cwd(&Path::new(&specifier))?;

  let permissions = state.borrow_mut::<PermissionsContainer>();
  permissions.check_read(&resolved_path)?;

  Ok(std::fs::read(resolved_path)?.into())
}
